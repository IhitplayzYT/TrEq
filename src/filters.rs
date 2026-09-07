pub mod filters{
    use std::{cell::RefCell, collections::{HashMap, binary_heap}, rc::Rc};

use uuid::Uuid;

use crate::models::models::EqFilter;

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
        pub fn new(filter: EqFilter) -> Self{
            Self { id: Uuid::new_v4(), biquad: Biquad::new(filter) }
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
        pub fn new(filter: EqFilter) -> Self{
            Self { x_1: 0.0, x_2: 0.0, y_1: 0.0, y_2: 0.0, b0: 0.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0}
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


        pub fn layer_biquad_pass(mut x:f32,biquads:& Vec<Biquad>) -> f32{
            for i in biquads{
                x = i.biquad_pass(x);
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