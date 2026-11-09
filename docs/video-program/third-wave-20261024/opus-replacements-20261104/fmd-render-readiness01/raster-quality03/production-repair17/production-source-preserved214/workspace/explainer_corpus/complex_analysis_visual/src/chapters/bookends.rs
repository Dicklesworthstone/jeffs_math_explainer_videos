//! Draft native FrankenManim complex-plane bookends.
//! Unintegrated, uncompiled, unrendered, unlistened and unreviewed.
//! 00_1 and 07_1 are proposed text cue boundaries, not verified audio cuts.
use explainer_kit::{Kit, Palette, Timed, hex, ink_style, stroke_style, v};
use fmn::prelude::*;

pub struct Opening { pub kit: Kit }
pub struct Closing { pub kit: Kit }

/// Map one complex point through z²; shared by the grid and moving point.
fn square(x: f64, y: f64) -> (f64, f64) { (x*x-y*y, 2.0*x*y) }

fn appear(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation=fade_in(stage.arena_mut(),mob,ORIGIN,1.0)?;
        animations.push(stage.prepare(animation.rt(0.5))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

fn disappear(stage: &mut Stage<'_>, mobs: &[Mob]) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation=fade_out(stage.arena_mut(),mob,ORIGIN,1.0)?;
        animations.push(stage.prepare(animation.rt(0.5))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

impl SceneConstruct for Opening {
    fn name(&self) -> &str { "00_opening" }
    fn construct(&mut self,stage:&mut Stage<'_>)->fmn::Result<()> {
        let kit=self.kit.clone();
        let input=(-3.0_f64,0.0_f64);let output=(3.0_f64,0.0_f64);let scale=2.0_f64;
        let (x,y)=(0.48_f64,0.36_f64);let (u,w)=square(x,y);
        kit.say(stage,"00_1")?;
        let mut real=vec![kit.text(stage,"Real function: line to line",34.0,Palette::ink(),v(0.0,2.9))?];
        for center in [input,output] {
            real.push(stage.arena_mut().add(Line::new(v(center.0-1.85,center.1),v(center.0+1.85,center.1)).style(stroke_style(Palette::dim(),2.0)).build()?));
        }
        real.push(stage.arena_mut().add(Circle::new().radius(0.13).style(ink_style(Palette::blue())).build().moved_to(v(input.0+1.0,input.1))));
        real.push(stage.arena_mut().add(Circle::new().radius(0.13).style(ink_style(Palette::green())).build().moved_to(v(output.0+1.0,output.1))));
        appear(stage,&real)?;
        // These sentence marks are estimates until the whole take is heard.
        kit.until_sentence(stage,"00_1",1)?;
        disappear(stage,&real)?;
        let title=kit.text(stage,"A function from plane to plane",34.0,Palette::ink(),v(0.0,2.9))?;
        let left=kit.text(stage,"input z",24.0,Palette::blue(),v(input.0,2.2))?;
        let right=kit.text(stage,"output f(z)",24.0,Palette::green(),v(output.0,2.2))?;
        let mut picture=vec![title,left,right];
        for center in [input,output] {
            picture.push(stage.arena_mut().add(Line::new(v(center.0-1.85,center.1),v(center.0+1.85,center.1)).style(stroke_style(Palette::dim(),2.0)).build()?));
            picture.push(stage.arena_mut().add(Line::new(v(center.0,center.1-1.85),v(center.0,center.1+1.85)).style(stroke_style(Palette::dim(),2.0)).build()?));
        }
        // Corresponding input grid lines and their z² images, sampled as
        // short segments. The chosen square stays inside both displayed axes.
        let half=0.7_f64;
        let divisions=32_u32;
        for family in 0..2 {
            let color=if family==0 { hex("#58B9CA") } else { hex("#DFB65C") };
            for fixed in [-0.6_f64,-0.3,0.0,0.3,0.6] {
                let coord=|t:f64| if family==0 {(fixed,t)} else {(t,fixed)};
                let (ax,ay)=coord(-half);let (bx,by)=coord(half);
                picture.push(stage.arena_mut().add(Line::new(v(input.0+scale*ax,input.1+scale*ay),v(input.0+scale*bx,input.1+scale*by)).style(stroke_style(color,2.6)).build()?));
                for k in 0..divisions {
                    let t0=-half+2.0*half*f64::from(k)/f64::from(divisions);
                    let t1=-half+2.0*half*f64::from(k+1)/f64::from(divisions);
                    let (x0,y0)=coord(t0);let (x1,y1)=coord(t1);
                    let (u0,v0)=square(x0,y0);let (u1,v1)=square(x1,y1);
                    picture.push(stage.arena_mut().add(Line::new(v(output.0+scale*u0,output.1+scale*v0),v(output.0+scale*u1,output.1+scale*v1)).style(stroke_style(color,2.6)).build()?));
                }
            }
        }
        let a=stage.arena_mut().add(Circle::new().radius(0.13).style(ink_style(Palette::blue())).build().moved_to(v(input.0+scale*x,input.1+scale*y)));
        let b=stage.arena_mut().add(Circle::new().radius(0.13).style(ink_style(Palette::green())).build().moved_to(v(output.0+scale*u,output.1+scale*w)));
        let example=kit.text(stage,"Example: f(z) = z²",25.0,Palette::hero(),v(0.0,-2.5))?;
        appear(stage,&picture)?;appear(stage,&[a])?;
        kit.until_sentence(stage,"00_1",2)?;
        appear(stage,&[b,example])?;
        // Move the input along a radius-0.6 arc inside the grid and compute z² at every
        // step, preserving a visible one-to-one source/output relationship.
        // Sentence timing remains provisional until the take is heard.
        kit.until_sentence(stage,"00_1",3)?;
        let start_angle=y.atan2(x);
        for step in 1..=24 {
            let theta=start_angle+std::f64::consts::FRAC_PI_2*f64::from(step)/24.0;
            let (ix,iy)=(0.6*theta.cos(),0.6*theta.sin());
            let (ox,oy)=square(ix,iy);
            let first=a.animate().set_anim_args(AnimateArgs { run_time: Some(0.16), ..AnimateArgs::default() })?.move_to(v(input.0+scale*ix,input.1+scale*iy),ORIGIN)?;
            let first=stage.prepare(first)?;
            let second=b.animate().set_anim_args(AnimateArgs { run_time: Some(0.16), ..AnimateArgs::default() })?.move_to(v(output.0+scale*ox,output.1+scale*oy),ORIGIN)?;
            let second=stage.prepare(second)?;
            stage.play_prepared(vec![first,second])?;
        }
        kit.finish(stage)
    }
}

impl SceneConstruct for Closing {
    fn name(&self) -> &str { "07_closing" }
    fn construct(&mut self,stage:&mut Stage<'_>)->fmn::Result<()> {
        let kit=self.kit.clone();kit.say(stage,"07_1")?;
        let title=kit.text(stage,"One local rule, far-reaching consequences",32.0,Palette::ink(),v(0.0,2.8))?;
        let a=kit.text(stage,"Derivative: one scale and rotation",26.0,Palette::blue(),v(0.0,1.1))?;
        let b=kit.text(stage,"Boundary: values determine the interior",26.0,Palette::green(),v(0.0,0.1))?;
        let c=kit.text(stage,"Contours: turns count zeros and poles",26.0,Palette::hero(),v(0.0,-0.9))?;
        let d=kit.text(stage,"Color shows direction; geometry gives it meaning",24.0,Palette::ink(),v(0.0,-2.1))?;
        appear(stage,&[title])?;appear(stage,&[a])?;appear(stage,&[b])?;appear(stage,&[c])?;appear(stage,&[d])?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::square;
    #[test]
    fn moving_point_stays_within_sampled_input_grid() {
        for step in 0..=24 {
            let theta=0.36_f64.atan2(0.48)+std::f64::consts::FRAC_PI_2*f64::from(step)/24.0;
            let (x,y)=(0.6*theta.cos(),0.6*theta.sin());
            let (u,v)=square(x,y);
            assert!(x.abs()<=0.7 && y.abs()<=0.7);
            assert!((u.hypot(v)-0.36).abs()<1e-12);
        }
    }
    #[test]
    fn displayed_grid_and_its_image_fit_the_axes() {
        for fixed in [-0.6_f64,-0.3,0.0,0.3,0.6] {
            for k in 0..=32 {
                let t=-0.7+1.4*f64::from(k)/32.0;
                for (x,y) in [(fixed,t),(t,fixed)] {
                    let (u,v)=square(x,y);
                    assert!(2.0*x.abs()<=1.85 && 2.0*y.abs()<=1.85);
                    assert!(2.0*u.abs()<=1.85 && 2.0*v.abs()<=1.85);
                }
            }
        }
    }
    #[test]
    fn square_grid_and_point_use_the_same_map() {
        for x in [-0.7_f64,-0.3,0.0,0.3,0.7] {
            for y in [-0.7_f64,-0.3,0.0,0.3,0.7] {
                let (u,v)=square(x,y);let (other_u,other_v)=square(-x,-y);
                assert!((u-other_u).abs()<1e-12 && (v-other_v).abs()<1e-12);
                assert!((u*u+v*v-(x*x+y*y).powi(2)).abs()<1e-12);

            }
        }
    }
}
