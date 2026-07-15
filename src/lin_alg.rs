use crate::sparse_matrix::SparseMatrix;

/*
 * Measured on a synthetic 12k x 80k corpus, relative sigma error vs a fully
 * converged run: 5e-3 at 8 rounds, 9e-4 at 12, 2e-4 at 14.
 */
const MAX_BLOCK_ITERATIONS: usize = 14;

/*
 * Stop iterating when every tracked sigma^2 estimate changes less than this
 * (relative) between rounds.
 */
const CONVERGENCE_THRESHOLD: f64 = 1e-3;

/*
 * Extra throwaway columns added to the block. They keep the singular
 * directions right below the requested ones busy, so the columns we keep
 * stop competing against near-equal neighbours and converge in fewer
 * rounds. Dropped at the end of rayleigh_ritz().
 */
const OVERSAMPLE: usize = 8;

/*
 * Jacobi stops when every off-diagonal entry is below JACOBI_EPS, which
 * is close enough to zero to call the matrix diagonal. The rotation cap
 * is a safety net, a block sized matrix needs far fewer.
 */
const JACOBI_MAX_ROTATIONS: usize = 10_000;
const JACOBI_EPS: f64 = 1e-12;

fn norm(v: &[f64]) -> f64 {
    v.iter().map(|x| x * x).sum::<f64>().sqrt()
}

fn normalize(v: &mut [f64]) {
    let n = norm(v);

    if n > 0.0 {
        for x in v.iter_mut() {
            *x /= n;
        }
    }
}

fn init_vector(size: usize, dim: usize) -> Vec<f64> {
    let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;

    /*
     * These magic numbers are just because.
     */
    let offset = dim * 997 + 7;
    let mut vector: Vec<f64> = (0..size)
        .map(|i| ((i + offset) as f64 * phi).fract() - 0.5)
        .collect();

    normalize(&mut vector);
    vector
}

/*
 * Blocks are stored row-major: the block_size values of row r live at
 * q[r * block_size .. (r + 1) * block_size]. Column j of the block is
 * the j-th candidate singular vector.
 */
fn init_block(rows: usize, block_size: usize) -> Vec<f64> {
    let mut q = vec![0.0; rows * block_size];

    for j in 0..block_size {
        let column = init_vector(rows, j);

        for (r, row) in q.chunks_exact_mut(block_size).enumerate() {
            row[j] = column[r];
        }
    }
    q
}

fn col_dot(q: &[f64], block_size: usize, i: usize, j: usize) -> f64 {
    q.chunks_exact(block_size).map(|row| row[i] * row[j]).sum()
}

/*
 * Modified Gram-Schmidt over the columns of the block.
 *
 * If a column collapses (it was linearly dependent on the previous
 * ones) it gets reseeded deterministically and projected again.
 */
fn orthonormalize(q: &mut [f64], block_size: usize) {
    let rows = q.len() / block_size;

    for j in 0..block_size {
        let mut reseeded = false;

        loop {
            for i in 0..j {
                let overlap = col_dot(q, block_size, i, j);

                /*
                 * Remove i projection from j.
                 */
                for row in q.chunks_exact_mut(block_size) {
                    row[j] -= overlap * row[i];
                }
            }

            /*
             * Normalize.
             */
            let n = col_dot(q, block_size, j, j).sqrt();
            if n > 1e-12 {
                for row in q.chunks_exact_mut(block_size) {
                    row[j] /= n;
                }
                break;
            }

            /*
             * Only try once to find a non-linear dependent vector.
             */
            if reseeded {
                break;
            }
            reseeded = true;

            /*
             * Generate a new vector and try again.
             */
            let seed = init_vector(rows, block_size + j);
            for (r, row) in q.chunks_exact_mut(block_size).enumerate() {
                row[j] = seed[r];
            }
        }
    }
}

/*
 * Rayleigh quotients of every block column:
 *
 *   rq[j] = q_j . (A * A^t * q_j)
 *
 * each one estimates the sigma^2 of its column. Sorted biggest first
 * because columns with near-equal values swap places between rounds:
 * the per-column values jump around but the sorted list is stable,
 * which is what the convergence check needs.
 */
fn ritz_values(q: &[f64], z: &[f64], block_size: usize) -> Vec<f64> {
    let mut rq = vec![0.0; block_size];

    let row_pairs = q.chunks_exact(block_size).zip(z.chunks_exact(block_size));
    for (qrow, zrow) in row_pairs {
        for j in 0..block_size {
            rq[j] += qrow[j] * zrow[j];
        }
    }

    rq.sort_by(|a, b| b.total_cmp(a));
    rq
}

/*
 * Only the first tracked values need to settle, the oversampled ones are
 * allowed to keep moving.
 */
fn ritz_converged(rq: &[f64], prev: &[f64], tracked: usize) -> bool {
    rq.iter()
        .zip(prev.iter())
        .take(tracked)
        .all(|(&a, &b)| (a - b).abs() <= CONVERGENCE_THRESHOLD * a.abs().max(1e-30))
}

struct EigenDecomposition {
    /*
     * Sorted biggest first, values[k] pairs with vectors[k].
     */
    values: Vec<f64>,
    vectors: Vec<Vec<f64>>,
}

/*
 * Eigen decomposition of a small symmetric matrix via classic Jacobi
 * rotations: find the largest off-diagonal entry, zero it with a 2x2
 * rotation, repeat until the matrix is diagonal.
 *
 * Only works on symmetric matrices, which T always is here.
 *
 * p and q are the standard names for the pivot indices in Jacobi.
 */
fn jacobi_eigen(mut t: Vec<Vec<f64>>) -> EigenDecomposition {
    let n = t.len();

    let mut rotations = vec![vec![0.0; n]; n];
    for i in 0..n {
        rotations[i][i] = 1.0;
    }

    for _ in 0..JACOBI_MAX_ROTATIONS {
        let mut p = 0;
        let mut q = 0;
        let mut max = 0.0;

        for i in 0..n {
            for j in (i + 1)..n {
                if t[i][j].abs() > max {
                    max = t[i][j].abs();
                    p = i;
                    q = j;
                }
            }
        }

        if max < JACOBI_EPS {
            break;
        }

        /*
         * Rotation angle that zeroes t[p][q], written in the stable
         * form (Golub & Van Loan) to avoid overflow when the diagonal
         * dominates.
         */
        let theta = (t[q][q] - t[p][p]) / (2.0 * t[p][q]);
        let tan = theta.signum() / (theta.abs() + (theta * theta + 1.0).sqrt());
        let cos = 1.0 / (tan * tan + 1.0).sqrt();
        let sin = tan * cos;

        for i in 0..n {
            let (a, b) = (t[i][p], t[i][q]);
            t[i][p] = cos * a - sin * b;
            t[i][q] = sin * a + cos * b;
        }

        for i in 0..n {
            let (a, b) = (t[p][i], t[q][i]);
            t[p][i] = cos * a - sin * b;
            t[q][i] = sin * a + cos * b;
        }

        for i in 0..n {
            let (a, b) = (rotations[i][p], rotations[i][q]);
            rotations[i][p] = cos * a - sin * b;
            rotations[i][q] = sin * a + cos * b;
        }
    }

    /*
     * Pack each eigenvalue (diagonal of t) with its eigenvector (the
     * matching column of the accumulated rotations) and sort the pairs
     * together, biggest first: they can never get mismatched.
     */
    let mut pairs: Vec<(f64, Vec<f64>)> = (0..n)
        .map(|k| (t[k][k], (0..n).map(|i| rotations[i][k]).collect()))
        .collect();

    pairs.sort_by(|a, b| b.0.total_cmp(&a.0));

    let (values, vectors): (Vec<f64>, Vec<Vec<f64>>) = pairs.into_iter().unzip();

    EigenDecomposition { values, vectors }
}

/*
 * After the loop the block spans the right subspace but each column still
 * blends several singular vectors. To untangle them, build T, a
 * block×block table of dot products:
 *
 *   T[i][j] = q_i . (A * A^t * q_j)
 *
 * T[i][i] is the sigma^2 estimate of column i. T[i][j] with i != j is
 * how mixed columns i and j still are, 0 means fully untangled.
 */
fn rayleigh_ritz(
    matrix: &SparseMatrix,
    q: &[f64],
    block_size: usize,
    dims: usize,
) -> (Vec<Vec<f64>>, Vec<f64>) {
    let rows = matrix.rows;
    let cols = matrix.cols;

    let mut buffer = vec![0.0; cols * block_size];
    let mut z = vec![0.0; rows * block_size];

    matrix.mul_block_t(q, &mut buffer, block_size);
    matrix.mul_block(&buffer, &mut z, block_size);

    let mut t = vec![vec![0.0; block_size]; block_size];

    let row_pairs = q.chunks_exact(block_size).zip(z.chunks_exact(block_size));
    for (qrow, zrow) in row_pairs {
        for i in 0..block_size {
            for j in i..block_size {
                t[i][j] += qrow[i] * zrow[j];
            }
        }
    }

    /*
     * T is symmetric because A * A^t is, mirror the upper triangle.
     */
    for i in 0..block_size {
        for j in 0..i {
            t[i][j] = t[j][i];
        }
    }

    let eig = jacobi_eigen(t);

    /*
     * Eigenpairs come sorted biggest first: keep the requested
     * dimensions and drop the oversampled ones.
     */
    let keep = dims.min(block_size);
    let mut vectors: Vec<Vec<f64>> = Vec::with_capacity(keep);
    let mut sigmas: Vec<f64> = Vec::with_capacity(keep);

    for col in 0..keep {
        let lambda = eig.values[col];

        /*
         * lambda is sigma^2, skip near-zero singular values.
         */
        if lambda < 1e-16 {
            break;
        }

        let eigvec = &eig.vectors[col];
        let mut u = vec![0.0; rows];

        for (r, qrow) in q.chunks_exact(block_size).enumerate() {
            u[r] = qrow.iter().zip(eigvec).map(|(a, b)| a * b).sum();
        }

        vectors.push(u);
        sigmas.push(lambda.sqrt());
    }

    (vectors, sigmas)
}

pub fn truncated_svd(
    importance_matrix: &SparseMatrix,
    dims: usize,
) -> (Vec<Vec<f64>>, Vec<f64>) {
    let rows = importance_matrix.rows;
    let cols = importance_matrix.cols;

    /*
     * Iterate a few extra columns but never more than the matrix shape allows.
     */
    let block_size = (dims + OVERSAMPLE).min(rows).min(cols);
    if block_size == 0 || dims == 0 {
        return (Vec::new(), Vec::new());
    }

    /*
     * Only the columns we will keep need to converge, the oversampled
     * ones are noise absorbers.
     */
    let tracked = dims.min(block_size);

    let mut q = init_block(rows, block_size);
    let mut buffer = vec![0.0; cols * block_size];
    let mut z = vec![0.0; rows * block_size];
    let mut prev_rq = vec![0.0; block_size];

    orthonormalize(&mut q, block_size);

    /*
     * Make Q an orthonormal base aligned with the dominant directions
     * of A:
     *
     *   Q <- orth(A * A^t * Q)
     */
    for _ in 0..MAX_BLOCK_ITERATIONS {
        /*
         * To get A*A^t*Q make A*(A^t*Q).
         * This means many less operations.
         */
        importance_matrix.mul_block_t(&q, &mut buffer, block_size);
        importance_matrix.mul_block(&buffer, &mut z, block_size);

        let rq = ritz_values(&q, &z, block_size);

        q.copy_from_slice(&z);
        orthonormalize(&mut q, block_size);

        if ritz_converged(&rq, &prev_rq, tracked) {
            break;
        }

        prev_rq = rq;
    }

    rayleigh_ritz(importance_matrix, &q, block_size, dims)
}

