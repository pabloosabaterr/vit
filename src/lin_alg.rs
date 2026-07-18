use crate::sparse_matrix::SparseMatrix;

const MAX_BLOCK_ITERATIONS: usize = 14;
const CONVERGENCE_THRESHOLD: f32 = 1e-3;
const OVERSAMPLE: usize = 8;
const JACOBI_MAX_ROTATIONS: usize = 10_000;
const JACOBI_EPS: f32 = 1e-12;
const DENSE_CHUNK_ROWS: usize = 2048;

fn norm(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

fn normalize(v: &mut [f32]) {
    let n = norm(v);

    if n > 0.0 {
        for x in v.iter_mut() {
            *x /= n;
        }
    }
}

fn init_vector(size: usize, dim: usize) -> Vec<f32> {
    let phi = (1.0 + 5.0_f32.sqrt()) / 2.0;

    let offset = dim * 997 + 7;
    let mut vector: Vec<f32> = (0..size)
        .map(|i| ((i + offset) as f32 * phi).fract() - 0.5)
        .collect();

    normalize(&mut vector);
    vector
}

fn init_block(rows: usize, block_size: usize) -> Vec<f32> {
    let mut q = vec![0.0; rows * block_size];

    for j in 0..block_size {
        let column = init_vector(rows, j);

        for (r, row) in q.chunks_exact_mut(block_size).enumerate() {
            row[j] = column[r];
        }
    }
    q
}

fn col_dot(q: &[f32], block_size: usize, i: usize, j: usize) -> f32 {
    q.chunks_exact(block_size).map(|row| row[i] * row[j]).sum()
}

/*
 * G = Q^t * Z as a dense block_size×block_size matrix (row-major).
 *
 * Both blocks are row-major, so each thread streams its rows
 * contiguously and accumulates a private G, no sharing, no locks.
 * Only the upper triangle is computed, then mirrored: Z is always
 * (A * A^t) * Q here so G is symmetric.
 */
fn gram(q: &[f32], z: &[f32], block_size: usize) -> Vec<f32> {
    let chunk = DENSE_CHUNK_ROWS * block_size;
    let mut partials: Vec<Vec<f32>> = Vec::new();

    std::thread::scope(|scope| {
        let mut handles = Vec::new();

        for (qc, zc) in q.chunks(chunk).zip(z.chunks(chunk)) {
            handles.push(scope.spawn(move || {
                let mut g = vec![0.0; block_size * block_size];

                let rows = qc.chunks_exact(block_size)
                    .zip(zc.chunks_exact(block_size));
                for (qrow, zrow) in rows {
                    for (i, &qi) in qrow.iter().enumerate() {
                        let grow = &mut g[i * block_size..(i + 1) * block_size];

                        for j in i..block_size {
                            grow[j] += qi * zrow[j];
                        }
                    }
                }
                g
            }));
        }

        for handle in handles {
            partials.push(handle.join().unwrap());
        }
    });

    let mut g = vec![0.0; block_size * block_size];
    for partial in partials {
        for (acc, val) in g.iter_mut().zip(partial) {
            *acc += val;
        }
    }

    for i in 0..block_size {
        for j in 0..i {
            g[i * block_size + j] = g[j * block_size + i];
        }
    }
    g
}

/*
 * Cholesky factorization G = R^t * R with R upper triangular.
 *
 * Returns None when a pivot is not clearly positive, which means the
 * columns behind G are (numerically) linearly dependent and the
 * caller has to fall back to Gram-Schmidt, which knows how to reseed
 * them.
 */
fn cholesky_upper(g: &[f32], block_size: usize) -> Option<Vec<f32>> {
    let mut r = vec![0.0; block_size * block_size];

    let mut max_diag = 0.0_f32;
    for i in 0..block_size {
        max_diag = max_diag.max(g[i * block_size + i]);
    }

    for j in 0..block_size {
        let mut diag = g[j * block_size + j];

        for i in 0..j {
            diag -= r[i * block_size + j] * r[i * block_size + j];
        }

        if !diag.is_finite() || diag <= max_diag * 1e-15 {
            return None;
        }

        let pivot = diag.sqrt();
        r[j * block_size + j] = pivot;

        for l in (j + 1)..block_size {
            let mut sum = g[j * block_size + l];

            for i in 0..j {
                sum -= r[i * block_size + j] * r[i * block_size + l];
            }
            r[j * block_size + l] = sum / pivot;
        }
    }
    Some(r)
}

/*
 * Q <- Q * R^-1, row by row.
 *
 * For one row q of Q the new row x solves x * R = q, and since R is
 * upper triangular that is a forward substitution:
 *
 *   x[j] = (q[j] - sum_{i<j} x[i] * R[i][j]) / R[j][j]
 *
 * done in place (x[i] for i < j is already the new value). Rows are
 * independent so they split across threads like everything else.
 *
 * R gets transposed first so the inner sum walks contiguous memory.
 */
fn solve_rows(q: &mut [f32], r: &[f32], block_size: usize) {
    let mut rt = vec![0.0; block_size * block_size];
    for i in 0..block_size {
        for j in i..block_size {
            rt[j * block_size + i] = r[i * block_size + j];
        }
    }
    let rt = &rt;

    std::thread::scope(|scope| {
        for chunk in q.chunks_mut(DENSE_CHUNK_ROWS * block_size) {
            scope.spawn(move || {
                for row in chunk.chunks_exact_mut(block_size) {
                    for j in 0..block_size {
                        let rrow = &rt[j * block_size..j * block_size + j];
                        let dot: f32 = row[..j]
                            .iter()
                            .zip(rrow)
                            .map(|(a, b)| a * b)
                            .sum();

                        row[j] = (row[j] - dot) / rt[j * block_size + j];
                    }
                }
            });
        }
    });
}

/*
 * Makes the block columns orthonormal via Cholesky QR:
 *
 *   G = Q^t * Q,  G = R^t * R,  Q <- Q * R^-1
 *
 * Same result as Gram-Schmidt but built from three cache-friendly,
 * parallel passes instead of column-by-column strided walks. When
 * the columns are too degenerate for Cholesky it falls back to the
 * Gram-Schmidt below, which reseeds collapsed columns.
 */
fn orthonormalize(q: &mut [f32], block_size: usize) {
    let g = gram(q, q, block_size);

    match cholesky_upper(&g, block_size) {
        Some(r) => solve_rows(q, &r, block_size),
        None => mgs_orthonormalize(q, block_size),
    }
}

fn mgs_orthonormalize(q: &mut [f32], block_size: usize) {
    let rows = q.len() / block_size;

    for j in 0..block_size {
        let mut reseeded = false;

        loop {
            for i in 0..j {
                let overlap = col_dot(q, block_size, i, j);

                for row in q.chunks_exact_mut(block_size) {
                    row[j] -= overlap * row[i];
                }
            }

            let n = col_dot(q, block_size, j, j).sqrt();
            if n > 1e-12 {
                for row in q.chunks_exact_mut(block_size) {
                    row[j] /= n;
                }
                break;
            }

            if reseeded {
                break;
            }
            reseeded = true;

            let seed = init_vector(rows, block_size + j);
            for (r, row) in q.chunks_exact_mut(block_size).enumerate() {
                row[j] = seed[r];
            }
        }
    }
}

fn ritz_values(q: &[f32], z: &[f32], block_size: usize) -> Vec<f32> {
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

fn ritz_converged(rq: &[f32], prev: &[f32], tracked: usize) -> bool {
    rq.iter()
        .zip(prev.iter())
        .take(tracked)
        .all(|(&a, &b)| (a - b).abs() <= CONVERGENCE_THRESHOLD * a.abs().max(1e-30))
}

struct EigenDecomposition {
    values: Vec<f32>,
    vectors: Vec<Vec<f32>>,
}

fn jacobi_eigen(mut t: Vec<Vec<f32>>) -> EigenDecomposition {
    let n = t.len();

    let mut rotations = vec![vec![0.0; n]; n];
    for (i, row) in rotations.iter_mut().enumerate().take(n) {
        row[i] = 1.0;
    }

    for _ in 0..JACOBI_MAX_ROTATIONS {
        let mut p = 0;
        let mut q = 0;
        let mut max = 0.0;

        for (i, row) in t.iter().enumerate().take(n) {
            for (j, val) in row.iter().enumerate().take(n).skip(i + 1) {
                if val.abs() > max {
                    max = val.abs();
                    p = i;
                    q = j;
                }
            }
        }

        if max < JACOBI_EPS {
            break;
        }

        let theta = (t[q][q] - t[p][p]) / (2.0 * t[p][q]);
        let tan = theta.signum() / (theta.abs() + (theta * theta + 1.0).sqrt());
        let cos = 1.0 / (tan * tan + 1.0).sqrt();
        let sin = tan * cos;

        for row in t.iter_mut().take(n) {
            let (a, b) = (row[p], row[q]);
            row[p] = cos * a - sin * b;
            row[q] = sin * a + cos * b;
        }

        let (left, right) = t.split_at_mut(q);
        for (a, b) in left[p].iter_mut().zip(right[0].iter_mut()) {
            let (va, vb) = (*a, *b);
            *a = cos * va - sin * vb;
            *b = sin * va + cos * vb;
        }

        for row in rotations.iter_mut().take(n) {
            let (a, b) = (row[p], row[q]);
            row[p] = cos * a - sin * b;
            row[q] = sin * a + cos * b;
        }
    }

    let mut pairs: Vec<(f32, Vec<f32>)> = (0..n)
        .map(|k| (t[k][k], (0..n).map(|i| rotations[i][k]).collect()))
        .collect();

    pairs.sort_by(|a, b| b.0.total_cmp(&a.0));

    let (values, vectors): (Vec<f32>, Vec<Vec<f32>>) = pairs.into_iter().unzip();

    EigenDecomposition { values, vectors }
}

fn rayleigh_ritz(
    matrix: &SparseMatrix,
    q: &[f32],
    block_size: usize,
    dims: usize,
) -> (Vec<Vec<f32>>, Vec<f32>) {
    let rows = matrix.rows;
    let cols = matrix.cols;

    let mut buffer = vec![0.0; cols * block_size];
    let mut z = vec![0.0; rows * block_size];

    matrix.mul_block_t(q, &mut buffer, block_size);
    matrix.mul_block(&buffer, &mut z, block_size);

    /*
     * T is the same Q^t * (A * A^t * Q) product as an orthonormalize
     * gram, reuse the parallel version and unpack it row by row.
     */
    let t: Vec<Vec<f32>> = gram(q, &z, block_size)
        .chunks_exact(block_size)
        .map(|row| row.to_vec())
        .collect();

    let eig = jacobi_eigen(t);

    let keep = dims.min(block_size);
    let mut vectors: Vec<Vec<f32>> = Vec::with_capacity(keep);
    let mut sigmas: Vec<f32> = Vec::with_capacity(keep);

    for col in 0..keep {
        let lambda = eig.values[col];

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
) -> (Vec<Vec<f32>>, Vec<f32>) {
    let rows = importance_matrix.rows;
    let cols = importance_matrix.cols;

    let block_size = (dims + OVERSAMPLE).min(rows).min(cols);
    if block_size == 0 || dims == 0 {
        return (Vec::new(), Vec::new());
    }

    let tracked = dims.min(block_size);

    let mut q = init_block(rows, block_size);
    let mut buffer = vec![0.0; cols * block_size];
    let mut z = vec![0.0; rows * block_size];
    let mut prev_rq = vec![0.0; block_size];

    orthonormalize(&mut q, block_size);

    for _ in 0..MAX_BLOCK_ITERATIONS {
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

