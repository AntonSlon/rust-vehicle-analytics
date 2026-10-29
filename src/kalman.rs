use nalgebra::{SMatrix, SVector, Vector3};

use crate::bbox::{Bbox, bbox_cx_cy, bbox_to_measurement};

pub struct KalmanFilter {
    // изменяется
    x: SVector<f32, 8>, // вектор состояния
    // изменяется
    p: SMatrix<f32, 8, 8>, // матрица насколько сильно мы доверяем данным вектора состояния
    q: SMatrix<f32, 8, 8>, // шум процесса
    f: SMatrix<f32, 8, 8>, // физика процесса
    r: SMatrix<f32, 4, 4>, // шум измерений
    h: SMatrix<f32, 4, 8>,
    dt: f64, //
}

impl KalmanFilter {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        let cx = x + w / 2.0;
        let cy = y + h / 2.0;

        let dt = 0.001;
        Self {
            x: SVector::from([cx, cy, w, h, 0.0, 0.0, 0.0, 0.0]),
            p: SMatrix::from_diagonal(&SVector::from([
                10.0, 10.0, 10.0, 10.0, 1000.0, 1000.0, 1000.0, 1000.0,
            ])),
            q: SMatrix::from_diagonal(&SVector::from([0.01, 0.01, 0.01, 0.01, 1.0, 1.0, 1.0, 1.0])),
            r: SMatrix::from_diagonal(&SVector::from([25.0, 25.0, 1e9, 1e9])),
            f: SMatrix::from_row_slice(&[
                1.0, 0.0, 0.0, 0.0, dt, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, dt, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0, 0.0, dt, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, dt, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ]),
            h: SMatrix::<f32, 4, 8>::identity(),
            dt: 0.0,
        }
    }

    pub fn predict(&mut self) {
        self.x = self.f * self.x;
        self.p = self.f * self.p * self.f.transpose() + self.q;
    }

    pub fn update(&mut self, bbox: Bbox) {
        let cbbox = bbox_cx_cy(bbox);

        let z = bbox_to_measurement(cbbox);
        let y = z - self.h * self.x;
        let s = self.h * self.p * self.h.transpose() + self.r;
        // 4x8 * 8x8 * 8 x 4 + 8x8

        let s_inverse = if let Some(s) = s.try_inverse() {
            s
        } else {
            return;
        };

        let k = self.p * self.h.transpose() * s_inverse;
        self.x = self.x + k * y;
        let i = SMatrix::from_diagonal(&SVector::from([1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]));
        self.p = (i - k * self.h) * self.p;
    }

    pub fn f_update_dt(&mut self, dt: f64) {
        self.dt = dt
    }
}

// x = STATE
// Что я думаю об объекте
//
// P = STATE UNCERTAINTY
// Насколько я уверен в x
//
//
// F = MOTION MODEL
// Как x переходит в следующий момент
//
// Q = MOTION NOISE
// Насколько я не доверяю F
//
//
// z = MEASUREMENT
// Что пришло от detector
//
// H = MEASUREMENT MODEL
// Какие части x detector умеет видеть
//
// R = MEASUREMENT NOISE
// Насколько я не доверяю detector
//
//
// y = RESIDUAL
// measurement - prediction
//
// S = RESIDUAL UNCERTAINTY
// Насколько неопределён residual
//
// K = KALMAN GAIN
// Насколько сильно measurement исправляет prediction
