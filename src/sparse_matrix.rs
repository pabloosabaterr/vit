/*
 * Stores a words×commits matrix where most entries are zero.
 */
pub(crate) struct SparseMatrix {
    pub(crate) rows: usize,
    pub(crate) cols: usize,
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
