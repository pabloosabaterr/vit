/*
 * Stores a words×commits matrix where most entries are zero.
 */
pub struct SparseMatrix {
    pub rows: usize,
    pub cols: usize,
    row_ptr: Vec<usize>,
    col_idx: Vec<usize>,
    values: Vec<f64>,
}

impl SparseMatrix {
    pub fn from_triplets(
        rows: usize,
        cols: usize,
        triplets: &mut Vec<(usize, usize, f64)>,
    ) -> Self {
        triplets.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

        let mut row_ptr = vec![0usize; rows + 1];
        let mut col_idx = Vec::with_capacity(triplets.len());
        let mut values = Vec::with_capacity(triplets.len());

        for &(r, c, v) in triplets.iter() {
            row_ptr[r + 1] += 1;
            col_idx.push(c);
            values.push(v);
        }

        for i in 1..=rows {
            row_ptr[i] += row_ptr[i - 1];
        }

        SparseMatrix {
            rows,
            cols,
            row_ptr,
            col_idx,
            values,
        }
    }

    /*
     * y = A * x
     */
    pub fn mul_vec(&self, x: &[f64], y: &mut [f64]) {
        for v in y.iter_mut() {
            *v = 0.0;
        }
        for r in 0..self.rows {
            let mut sum = 0.0;
            for idx in self.row_ptr[r]..self.row_ptr[r + 1] {
                sum += self.values[idx] * x[self.col_idx[idx]];
            }
            y[r] = sum;
        }
    }

    /*
     * y = A^t * x
     */
    pub fn mul_vec_t(&self, x: &[f64], y: &mut [f64]) {
        for v in y.iter_mut() {
            *v = 0.0;
        }
        for r in 0..self.rows {
            let xr = x[r];
            if xr == 0.0 {
                continue;
            }
            for idx in self.row_ptr[r]..self.row_ptr[r + 1] {
                y[self.col_idx[idx]] += self.values[idx] * xr;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: &[f64], b: &[f64]) {
        assert_eq!(a.len(), b.len(), "length mismatch");
        for (&x, &y) in a.iter().zip(b.iter()) {
            assert!((x - y).abs() < 1e-10);
        }
    }

    /*
     * A simple 3x4 matrix:
     *
     *   [ 1.0  0.0  2.0  0.0 ]
     *   [ 0.0  3.0  0.0  0.0 ]
     *   [ 0.0  0.0  4.0  5.0 ]
     */
    fn make_3x4() -> SparseMatrix {
        let mut triplets = vec![
            (0, 0, 1.0),
            (0, 2, 2.0),
            (1, 1, 3.0),
            (2, 2, 4.0),
            (2, 3, 5.0),
        ];
        SparseMatrix::from_triplets(3, 4, &mut triplets)
    }

    #[test]
    fn mul_vec_basic() {
        let m = make_3x4();
        let x = vec![1.0, 2.0, 3.0, 4.0];
        let mut y = vec![0.0; 3];

        m.mul_vec(&x, &mut y);

        approx_eq(&y, &[7.0, 6.0, 32.0]);
    }

    #[test]
    fn mul_vec_t_basic() {
        let m = make_3x4();
        let x = vec![1.0, 2.0, 3.0];
        let mut y = vec![0.0; 4];

        m.mul_vec_t(&x, &mut y);

        approx_eq(&y, &[1.0, 6.0, 14.0, 15.0]);
    }
}
