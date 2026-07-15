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
     * Y = A * X
     *
     * X and Y are dense blocks of k columns stored row-major: the k
     * values of row r live at buf[r * k .. (r + 1) * k]. X is cols×k
     * and Y is rows×k.
     *
     * A single pass over the sparse matrix feeds every column. The
     * pass is memory bound so serving k columns instead of one is
     * almost free, and that is what makes block iteration fast.
     */
    pub fn mul_block(&self, x: &[f64], y: &mut [f64], k: usize) {
        for v in y.iter_mut() {
            *v = 0.0;
        }

        for r in 0..self.rows {
            let row = &mut y[r * k..(r + 1) * k];

            for idx in self.row_ptr[r]..self.row_ptr[r + 1] {
                let val = self.values[idx];
                let col = self.col_idx[idx];
                let src = &x[col * k..(col + 1) * k];

                for j in 0..k {
                    row[j] += val * src[j];
                }
            }
        }
    }

    /*
     * Y = A^t * X
     *
     * Same block layout as mul_block, X is rows×k and Y is cols×k.
     */
    pub fn mul_block_t(&self, x: &[f64], y: &mut [f64], k: usize) {
        for v in y.iter_mut() {
            *v = 0.0;
        }

        for r in 0..self.rows {
            let src = &x[r * k..(r + 1) * k];

            for idx in self.row_ptr[r]..self.row_ptr[r + 1] {
                let val = self.values[idx];
                let col = self.col_idx[idx];

                for j in 0..k {
                    y[col * k + j] += val * src[j];
                }
            }
        }
    }
}
