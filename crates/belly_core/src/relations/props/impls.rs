use std::marker::PhantomData;

use bevy::{color::{Alpha, Color, Hue}, reflect::Reflect};

use crate::ess::property::ColorFromHexExtension;

use crate::{
    build::{Prop, TransformationResult},
    impl_properties,
    relations::bind::{BindableSource, BindableTarget},
};

use super::{GetProperties, SetGet};

impl_properties! { ColorProperties for Color {
    r(set_r, r) => |v: f32| v.min(1.).max(0.);
    g(set_g, g) => |v: f32| v.min(1.).max(0.);
    b(set_b, b) => |v: f32| v.min(1.).max(0.);
    a(set_a, a) => |v: f32| v.min(1.).max(0.);
    one_minus_r(set_r, r) => |v: f32| (1.0 - v).min(1.).max(0.);
    one_minus_g(set_g, g) => |v: f32| (1.0 - v).min(1.).max(0.);
    one_minus_b(set_b, b) => |v: f32| (1.0 - v).min(1.).max(0.);
    one_minus_a(set_a, a) => |v: f32| (1.0 - v).min(1.).max(0.);
    hex(set_hex, get_hex) => |v: String| v.clone();
}}

trait ColorExt {
    fn set_r(&mut self, r: f32);
    fn r(&mut self) -> f32;

    fn set_g(&mut self, g: f32);
    fn g(&mut self) -> f32;

    fn set_b(&mut self, b: f32);
    fn b(&mut self) -> f32;

    fn set_a(&mut self, a: f32);
    fn a(&mut self) -> f32;
}
impl ColorExt for Color {
    fn set_r(&mut self, r: f32) {
        *self = bevy::prelude::Color::LinearRgba(self.to_linear().with_red(r));
    }
    
    fn r(&mut self) -> f32 {
        self.to_linear().red
    }
    
    fn set_g(&mut self, g: f32) {
        *self = bevy::prelude::Color::LinearRgba(self.to_linear().with_green(g));
    }
    
    fn g(&mut self) -> f32 {
        self.to_linear().green
    }
    
    fn set_b(&mut self, b: f32) {
        *self = bevy::prelude::Color::LinearRgba(self.to_linear().with_blue(b));
    }
    
    fn b(&mut self) -> f32 {
       self.to_linear().blue
    }
    
    fn set_a(&mut self, a: f32) {
        self.set_alpha(a);
    }
    
    fn a(&mut self) -> f32 {
        self.alpha()
    }
}



pub struct OptionProperties<T>(PhantomData<T>);
fn set_some<T: BindableSource + BindableTarget>(
    val: &T,
    mut prop: Prop<Option<T>>,
) -> TransformationResult {
    if Some(val) != prop.as_ref() {
        *prop = Some(val.clone());
    }
    Ok(())
}
fn get_some<T: BindableSource + BindableTarget>(prop: Prop<Option<T>>) -> T {
    if let Some(value) = prop.as_ref() {
        return value.clone();
    } else {
        panic!("Can't use OptionProperties.some to fetch empty value");
    }
}
impl<T: BindableSource + BindableTarget> OptionProperties<T> {
    pub fn some(&self) -> SetGet<Option<T>, T> {
        SetGet::new(set_some, get_some)
    }
}
impl<T> GetProperties for Option<T> {
    type Item = OptionProperties<T>;
    fn get_properties() -> &'static Self::Item {
        &OptionProperties(PhantomData)
    }
}
