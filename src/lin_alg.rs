use crate::sparse_matrix::SparseMatrix;

/*
 * Maximum iterations for convergece of the vector.
 */
const MAX_CONVERGENCE_ITERATIONS: usize = 80;
/*
 * Minimum change to consider a vector converged.
 */
const CONVERGENCE_THRESHOLD: f64 = 1e-6;

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

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
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

pub(crate) fn power_iteration(
    importance_matrix: &SparseMatrix,
    dims: usize,
) -> (Vec<Vec<f64>>, Vec<f64>) {
    let m = importance_matrix.rows;
    let n = importance_matrix.cols;
    let vector_nr = dims.min(m).min(n);

    let mut vectors: Vec<Vec<f64>> = Vec::with_capacity(vector_nr);
    let mut sigmas: Vec<f64> = Vec::with_capacity(vector_nr);

    let mut buffer = vec![0.0; n];
    let mut new_vector = vec![0.0; m];

    for vector in 0..vector_nr {
        let mut vector = init_vector(m, vector);

        for _ in 0..MAX_CONVERGENCE_ITERATIONS {
            /*
             * To get A*A^t*V make A*(A^t*V)
             * This means many less operations.
             */
            importance_matrix.mul_vec_t(&vector, &mut buffer);
            importance_matrix.mul_vec(&buffer, &mut new_vector);

            /* orthogonalize against previous vectors */
            for prev in &vectors {
                let overlap = dot(&new_vector, prev);
                for j in 0..m {
                    new_vector[j] -= overlap * prev[j];
                }
            }

            normalize(&mut new_vector);

            /*
             * Convergence check with sign-flip handling.
             * Eigenvectors can flip sign between iterations, checking both
             * orientations avoids false stalls.
             */
            let diff_pos: f64 = vector
                .iter()
                .zip(new_vector.iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                .sqrt();
            let diff_neg: f64 = vector
                .iter()
                .zip(new_vector.iter())
                .map(|(a, b)| (a + b).powi(2))
                .sum::<f64>()
                .sqrt();
            let diff = diff_pos.min(diff_neg);

            std::mem::swap(&mut vector, &mut new_vector);

            if diff < CONVERGENCE_THRESHOLD {
                break;
            }
        }

        /*
         * Measure how much variance this direction captures.
         * Higher sigma means this direction is more important.
         */
        importance_matrix.mul_vec_t(&vector, &mut buffer);
        let sigma = norm(&buffer);

        /*
         * Skip near-zero singular values.
         */
        if sigma < 1e-8 {
            break;
        }

        vectors.push(vector);
        sigmas.push(sigma);
    }

    (vectors, sigmas)
}
