pub mod filters{
    use std::{cell::RefCell, collections::{HashMap, binary_heap}, f64::consts::PI, rc::Rc};

use uuid::Uuid;

use crate::models::models::{EqFilter, Freq};

    pub struct BiquadMesh{
        // TODO: WAY TO COMPLEX of an idea make a NN style API to make Biquad Meshhes with Maxpool,AvgPool,SkipConnections and Dense layers
    }

    impl BiquadMesh{
    }


    pub struct BiquadNode{
        pub id: Uuid,
        pub biquad: Biquad,
    }

    impl BiquadNode{
        pub fn new(filter: EqFilter, freq: Freq, sample_rate: usize, q: f64, gain: Option<f64>) -> Self{
            Self { id: Uuid::new_v4(), biquad: Biquad::new(filter, freq, sample_rate, q, gain) }
        }

        pub fn process(&mut self, x: f32) -> f32 {
            self.biquad.biquad_pass_mut(x)
        }

        pub fn get_id(&self) -> Uuid {
            self.id
        }

        pub fn get_biquad(&self) -> &Biquad {
            &self.biquad
        }

        pub fn get_biquad_mut(&mut self) -> &mut Biquad {
            &mut self.biquad
        }
    }

    pub struct Biquad{
        x_1: f32,
        x_2: f32,
        y_1: f32,
        y_2: f32,
        pub b0: f32,
        pub b1: f32,
        pub b2: f32,
        pub a1: f32,
        pub a2: f32,
    }

    impl Biquad{
        pub fn new(filter: EqFilter,freq: Freq,sample_rate: usize,q: f64,gain: Option<f64>) -> Self{
            use EqFilter::*;
            
            let (b0, b1, b2, a0, a1, a2) = match freq{
                Freq::Point(f) => {
                    Self::calculate_coefficients(filter, f as f64, sample_rate as f64, q, gain.unwrap_or(0.0))
                },
                Freq::Range(start, end) => {
                    let center = (start + end) as f64 / 2.0;
                    Self::calculate_coefficients(filter, center, sample_rate as f64, q, gain.unwrap_or(0.0))
                }
            };

            let (b0, b1, b2, a1, a2) = if a0 != 0.0 {
                (b0 / a0, b1 / a0, b2 / a0, a1 / a0, a2 / a0)
            } else {
                (b0, b1, b2, a1, a2)
            };

            Self { x_1: 0.0, x_2: 0.0, y_1: 0.0, y_2: 0.0, b0, b1, b2, a1, a2}
        }

        pub fn reset(&mut self) {
            self.x_1 = 0.0;
            self.x_2 = 0.0;
            self.y_1 = 0.0;
            self.y_2 = 0.0;
        }

        pub fn set_coefficients(&mut self, b0: f32, b1: f32, b2: f32, a1: f32, a2: f32) {
            self.b0 = b0;
            self.b1 = b1;
            self.b2 = b2;
            self.a1 = a1;
            self.a2 = a2;
        }

        pub fn get_coeffs(&self) -> (f32, f32, f32, f32, f32) {
            (self.b0, self.b1, self.b2, self.a1, self.a2)
        }

        pub fn get_coeffs_mut(&mut self) -> (&mut f32,&mut f32,&mut f32,&mut f32,&mut f32) {
            (&mut self.b0,&mut self.b1,&mut self.b2,&mut self.a1,&mut self.a2)
        }

        fn calculate_coefficients(filter: EqFilter, freq: f64, sample_rate: f64, q: f64, gain: f64) -> (f32, f32, f32, f32, f32, f32) {
            use EqFilter::*;            
            let w0 = 2.0 * PI * freq / sample_rate;
            let c = w0.cos();
            let s = w0.sin();
            let alpha = s / (2.0 * q);
            
            let (b0, b1, b2, a0, a1, a2) = match filter {
                Peaking => {
                    let a = 10.0_f64.powf(gain / 40.0);
                    let alpha_a = alpha * a;
                    let alpha_a_inv = alpha / a;
                    (1.0 + alpha_a,-2.0 * c,1.0 - alpha_a,1.0 + alpha_a_inv,-2.0 * c,1.0 - alpha_a_inv)
                },
                LowShelf => {
                    let a = 10.0_f64.powf(gain / 40.0);
                    let sqrt_a2 = a.sqrt() + 1.0 / a.sqrt();
                    let sqrt_a2_minus_1 = sqrt_a2 - 1.0;
                    let sqrt_a2_plus_1 = sqrt_a2 + 1.0;
                    (a * (sqrt_a2_plus_1 - sqrt_a2_minus_1 * c + alpha),2.0 * a * (sqrt_a2_minus_1 - sqrt_a2_plus_1 * c),a * (sqrt_a2_plus_1 - sqrt_a2_minus_1 * c - alpha),sqrt_a2_plus_1 + sqrt_a2_minus_1 * c + alpha,-2.0 * (sqrt_a2_minus_1 + sqrt_a2_plus_1 * c),sqrt_a2_plus_1 + sqrt_a2_minus_1 * c - alpha)
                },
                HighShelf => {
                    let a = 10.0_f64.powf(gain / 40.0);
                    let sqrt_a2 = a.sqrt() + 1.0 / a.sqrt();
                    let sqrt_a2_minus_1 = sqrt_a2 - 1.0;
                    let sqrt_a2_plus_1 = sqrt_a2 + 1.0;
                    (a * (sqrt_a2_plus_1 + sqrt_a2_minus_1 * c + alpha),-2.0 * a * (sqrt_a2_minus_1 + sqrt_a2_plus_1 * c),a * (sqrt_a2_plus_1 + sqrt_a2_minus_1 * c - alpha),sqrt_a2_plus_1 - sqrt_a2_minus_1 * c + alpha,2.0 * (sqrt_a2_minus_1 - sqrt_a2_plus_1 * c),sqrt_a2_plus_1 - sqrt_a2_minus_1 * c - alpha)
                },
                LowPass => {((1.0 - c) / 2.0,1.0 - c,(1.0 - c) / 2.0,1.0 + alpha,-2.0 * c,1.0 - alpha)},
                HighPass => {((1.0 + c) / 2.0,-(1.0 + c),(1.0 + c) / 2.0,1.0 + alpha,-2.0 * c,1.0 - alpha)},
                BandPass => {(alpha,0.0,-alpha,1.0 + alpha,-2.0 * c,1.0 - alpha)},
                Notch => {(1.0,-2.0 * c,1.0,1.0 + alpha,-2.0 * c,1.0 - alpha)}
            };
            (b0 as f32, b1 as f32, b2 as f32, a0 as f32, a1 as f32, a2 as f32)
        }

        pub fn biquad_pass_mut(&mut self, x: f32) -> f32{
            let y = self.b0 * x + self.b1 * self.x_1 + self.b2 * self.x_2 - self.a1 * self.y_1 - self.a2 * self.y_2;
            self.x_2 = self.x_1;
            self.x_1  = x;
            self.y_2 = self.y_1;
            self.y_1 = y;
            y
        }

        pub fn biquad_pass(&self, x: f32) -> f32{
            self.b0 * x + self.b1 * self.x_1 + self.b2 * self.x_2 - self.a1 * self.y_1 - self.a2 * self.y_2
        }


        pub fn layer_biquad_pass(mut x:f32,biquads:&mut Vec<Biquad>) -> f32{
            for i in biquads{
                x = i.biquad_pass_mut(x);
            } 
            x
        }
    }


    type Biquad_weights = (f32,f32,f32,f32,f32);
    type Y_Past = (f32,f32);
    type X_Past = (f32,f32);

    pub fn layer_biquad_pass(mut x:f32,x_past: &Vec<X_Past>,y_past:&Vec<Y_Past>,biquads: &Vec<Biquad_weights>) -> f32{
        if (x_past.len() != y_past.len()) || (x_past.len() != biquads.len()){
            return 0.0;
        }
        for ((&(b0,b1,b2,a1,a2),&(x_1,x_2)),&(y_1,y_2)) in biquads.iter().zip(x_past).zip(y_past){
            x = biquad_pass(x, x_1, x_2, y_1, y_2, b0, b1, b2, a1, a2);
        }
        x
    }

    // A hypothetical accumulated biquad pass
    pub fn running_biquad_pass(mut x:f32,mut x_1: f32,mut x_2:f32,mut y_1:f32,mut y_2:f32,biquads: &Vec<Biquad_weights>) -> f32{
        let mut y = 0.0;
        for &(b0,b1,b2,a1,a2) in biquads{
            y = biquad_pass(x, x_1, x_2, y_1, y_2, b0, b1, b2, a1, a2);
            x_2 = x_1;
            x_1 = x;
            x = y;            
            y_2 = y_1;
            y_1 = y;
        }
        y
    }

    pub fn biquad_pass(x:f32,x_1:f32,x_2:f32,y_1:f32,y_2:f32,b0: f32,b1: f32,b2:f32,a1:f32,a2:f32) -> f32{
        b0 * x + b1 * x_1 + b2 * x_2 - a1 * y_1 - a2 * y_2
    }
}