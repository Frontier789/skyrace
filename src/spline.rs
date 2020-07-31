use glui::tools::Vec3;
use std::ops::Range;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
struct CubicPolinomial {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
    pub d: Vec3,
}

impl CubicPolinomial {
    pub fn eval(self, x: f32) -> Vec3 {
        ((self.a * x + self.b) * x + self.c) * x + self.d
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Spline {
    xs: Vec<f32>,
    ys: Vec<Vec3>,
    coefs: Vec<CubicPolinomial>,
}

impl Spline {
    #[allow(dead_code)]
    pub fn eval(&self, x: f32) -> Vec3 {
        for i in 1..self.xs.len() {
            if self.xs[i] >= x {
                return self.coefs[i - 1].eval(x);
            }
        }
        Vec3::zero()
    }

    pub fn points(&self) -> impl Iterator<Item = &Vec3> {
        self.ys.iter()
    }

    pub fn support(&self) -> Range<f32> {
        self.xs[0]..*self.xs.last().unwrap()
    }

    pub fn quantize(&self, n: usize) -> Vec<Vec3> {
        let mut i = 0;
        let s = self.support();
        let mut x = s.start;
        let step = (s.end - s.start) / ((n - 1) as f32);

        let mut pts = vec![];

        while pts.len() < n {
            pts.push(self.coefs[i].eval(x));

            x += step;
            while i + 1 < self.coefs.len() && x > self.xs[i + 1] {
                i += 1;
            }
        }

        pts
    }

    fn solve_tridiag(a: Vec<f32>, mut b: Vec<f32>, c: Vec<f32>, mut d: Vec<Vec3>) -> Vec<Vec3> {
        let n = a.len();
        let mut x = vec![Vec3::zero(); n + 1];

        for i in 0..n {
            let w = a[i] / b[i];
            b[i + 1] = b[i + 1] - w * c[i];
            d[i + 1] = d[i + 1] - w * d[i];
        }
        x[n] = d[n] / b[n];
        for i in (0..n).rev() {
            x[i] = (d[i] - c[i] * x[i + 1]) / b[i];
        }

        x
    }

    fn second_divided_differences(y: &Vec<Vec3>, x: &Vec<f32>) -> Vec<Vec3> {
        let n = y.len() - 1;

        let mut div_diffs = Vec::with_capacity(n + 1);

        let f_derivate_x0 = Vec3::new(-1.635, 0.0, 0.0);
        let f_derivate_xn = Vec3::new(-0.706, 0.0, 0.0);

        div_diffs.push(((y[1] - y[0]) / (x[1] - x[0]) - f_derivate_x0) / (x[1] - x[0]));

        for i in 0..n - 1 {
            let dd = (y[i + 2] - y[i + 1]) / (x[i + 2] - x[i + 1]) / (x[i + 2] - x[i + 0])
                - (y[i + 1] - y[i + 0]) / (x[i + 1] - x[i + 0]) / (x[i + 2] - x[i + 0]);
            div_diffs.push(dd);
        }

        div_diffs.push((f_derivate_xn - (y[n] - y[n - 1]) / (x[n] - x[n - 1])) / (x[n] - x[n - 1]));

        div_diffs
    }

    pub fn fit_cubic(x: Vec<f32>, y: Vec<Vec3>) -> Spline {
        // h[i] = x[i+1] - x[i]
        let h = x.windows(2).map(|w| w[1] - w[0]).collect::<Vec<f32>>();

        // mu[i] = h[i] / (h[i] + h[i+1])
        let mut mu = h
            .windows(2)
            .map(|w| w[0] / (w[1] + w[0]))
            .collect::<Vec<f32>>();
        mu.push(1.0);

        // lambda[i] = h[i+1] / (h[i] + h[i+1])
        let lambda = vec![1.0]
            .into_iter()
            .chain(h.windows(2).map(|w| w[1] / (w[1] + w[0])))
            .collect::<Vec<f32>>();

        let n = x.len() - 1;

        let diag = vec![2.0; n + 1];

        let div_diff2 = Self::second_divided_differences(&y, &x);

        #[allow(non_snake_case)]
        let M = Self::solve_tridiag(mu, diag, lambda, div_diff2)
            .into_iter()
            .map(|m| 6.0 * m)
            .collect::<Vec<_>>();

        let mut polis = vec![CubicPolinomial::default(); n];

        for i in 0..n {
            let p = &mut polis[i];

            p.a = (M[i + 1] - M[i]) / h[i] / 6.0;
            p.b = (x[i + 1] * M[i] - x[i] * M[i + 1]) / h[i] / 2.0;
            p.c = (x[i] * x[i] * M[i + 1] - x[i + 1] * x[i + 1] * M[i]) / 2 / h[i]
                + (M[i] - M[i + 1]) * h[i] / 6.0
                + (y[i + 1] - y[i]) / h[i];
            p.d = (x[i + 1] * x[i + 1] * x[i + 1] * M[i] - x[i] * x[i] * x[i] * M[i + 1])
                / 6.0
                / h[i]
                + (M[i + 1] * x[i] - M[i] * x[i + 1]) * h[i] / 6.0
                + (y[i] * x[i + 1] - y[i + 1] * x[i]) / h[i];
        }

        let s = Spline {
            coefs: polis,
            xs: x,
            ys: y,
        };

        s
    }

    #[allow(dead_code)]
    pub fn fit_linear(xs: Vec<f32>, ys: Vec<Vec3>) -> Spline {
        let mut polis = vec![];

        for i in 1..ys.len() {
            let p = ys[i - 1];
            let q = ys[i];
            let t0 = xs[i - 1];
            let t1 = xs[i];

            let dir = (q - p) / (t1 - t0);

            let poli = CubicPolinomial {
                a: Vec3::zero(),
                b: Vec3::zero(),
                c: dir,
                d: p - t0 * dir,
            };

            polis.push(poli);
        }

        Spline {
            xs,
            ys,
            coefs: polis,
        }
    }
}
