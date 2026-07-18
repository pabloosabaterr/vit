use std::thread;

/*
 * Stores a words×commits matrix where most entries are zero.
 */
pub struct SparseMatrix {
    pub rows: usize,
    pub cols: usize,

    /* r c */
    row_ptr: Vec<usize>,
    col_idx: Vec<usize>,
    values: Vec<f32>,

    /* c r */
    t_row_ptr: Vec<usize>,
    t_col_idx: Vec<usize>,
    t_values: Vec<f32>,
}

fn build_csr(
    rows: usize,
    triplets: &[(usize, usize, f32)],
) -> (Vec<usize>, Vec<usize>, Vec<f32>) {
    let mut row_ptr = vec![0usize; rows + 1];
    let mut col_idx = Vec::with_capacity(triplets.len());
    let mut values = Vec::with_capacity(triplets.len());

    for &(r, c, v) in triplets {
        row_ptr[r + 1] += 1;
        col_idx.push(c);
        values.push(v);
    }

    for i in 1..=rows {
        row_ptr[i] += row_ptr[i - 1];
    }

    (row_ptr, col_idx, values)
}

fn mul_rows(
    row_ptr: &[usize],
    col_idx: &[usize],
    values: &[f32],
    x: &[f32],
    y: &mut [f32],
    k: usize,
    first_row: usize,
) {
    for v in y.iter_mut() {
        *v = 0.0;
    }

    for (i, out) in y.chunks_exact_mut(k).enumerate() {
        let r = first_row + i;

        for idx in row_ptr[r]..row_ptr[r + 1] {
            let val = values[idx];
            let src = &x[(col_idx[idx] * k)..((col_idx[idx] + 1) * k)];

            for (j, s) in out.iter_mut().zip(src) {
                *j += val * s;
            }
        }
    }
}

fn mul_csr_block_with(
    row_ptr: &[usize],
    col_idx: &[usize],
    values: &[f32],
    x: &[f32],
    y: &mut [f32],
    k: usize,
    threads: usize,
) {
    let rows = row_ptr.len() - 1;

    let chunk_rows = rows.div_ceil(threads).max(1);

    if threads == 1 || rows < threads {
        mul_rows(row_ptr, col_idx, values, x, y, k, 0);
        return;
    }

    thread::scope(|scope| {
        for (i, chunk) in y.chunks_mut(chunk_rows * k).enumerate() {
            scope.spawn(move || {
                mul_rows(row_ptr, col_idx, values, x, chunk, k, i * chunk_rows);
            });
        }
    });
}

fn mul_csr_block(
    row_ptr: &[usize],
    col_idx: &[usize],
    values: &[f32],
    x: &[f32],
    y: &mut [f32],
    k: usize,
) {
    let threads = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    mul_csr_block_with(row_ptr, col_idx, values, x, y, k, threads)
}

impl SparseMatrix {
    pub fn from_triplets(
        rows: usize,
        cols: usize,
        triplets: &mut [(usize, usize, f32)],
    ) -> Self {
        triplets.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        let (row_ptr, col_idx, values) = build_csr(rows, triplets);

        let mut swapped: Vec<(usize, usize, f32)> =
            triplets.iter().map(|&(r, c, v)| (c, r, v)).collect();

        swapped.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        let (t_row_ptr, t_col_idx, t_values) = build_csr(cols, &swapped);

        SparseMatrix {
            rows,
            cols,
            row_ptr,
            col_idx,
            values,
            t_row_ptr,
            t_col_idx,
            t_values,
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
    pub fn mul_block(&self, x: &[f32], y: &mut [f32], k: usize) {
        mul_csr_block(&self.row_ptr, &self.col_idx, &self.values, x, y, k);
    }

    /*
     * Y = A^t * X
     *
     * Same block layout as mul_block, X is rows×k and Y is cols×k.
     */
    pub fn mul_block_t(&self, x: &[f32], y: &mut [f32], k: usize) {
        mul_csr_block(&self.t_row_ptr, &self.t_col_idx, &self.t_values, x, y, k);
    }
}
