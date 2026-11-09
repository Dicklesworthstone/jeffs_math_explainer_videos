//! Draft native FrankenManim phase-color sampling scene, not production.
//! Unintegrated, uncompiled, unrendered, unlistened and unreviewed.
//! 01_1 is a proposed text cue; 01_2 has its own complete retained takes.
//! Neither is verified by listening or timed alignment.
use explainer_kit::{Kit, Palette, Timed, hex, ink_style, rect, v};
use fmn::prelude::*;

pub struct DomainColor { pub kit: Kit }

fn fade(stage: &mut Stage<'_>, mobs: &[Mob], incoming: bool) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = if incoming { fade_in(stage.arena_mut(),mob,ORIGIN,1.0)? } else { fade_out(stage.arena_mut(),mob,ORIGIN,1.0)? };
        animations.push(stage.prepare(animation.rt(0.5))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

const COLORS: [&str;12] = ["#ED5B62","#F08A51","#E7B44E","#B7C75B","#71C979","#4DC3A5","#4ABACD","#579BE2","#7A83E7","#A373D8","#C66DB3","#E3658C"];

/// Phase only; None marks a zero or a point where the reciprocal is undefined.
fn phase(kind: u8, x: f64, y: f64) -> Option<f64> {
    let radius_squared=x*x+y*y;
    match kind {
        0 if radius_squared>0.0 => Some((2.0*x*y).atan2(x*x-y*y)),
        1 if radius_squared>0.0 => Some((-y).atan2(x)),
        2 => Some(y),
        3 => {
            let re=x*x*x-3.0*x*y*y-1.0;
            let im=3.0*x*x*y-y*y*y;
            if re.hypot(im)>1e-12 { Some(im.atan2(re)) } else { None }
        }
        4 => {
            let re=x.sin()*y.cosh();
            let im=x.cos()*y.sinh();
            if re.hypot(im)>1e-12 { Some(im.atan2(re)) } else { None }
        }
        5 => {
            // (z+1)/(z-1): the denominator is positive after rationalizing.
            let denominator=(x-1.0)*(x-1.0)+y*y;
            let re=radius_squared-1.0;
            let im= -2.0*y;
            if denominator>1e-12 && re.hypot(im)>1e-12 { Some(im.atan2(re)) } else { None }
        }
        6 => {
            let re=x.cos()*y.cosh();
            let im= -x.sin()*y.sinh();
            if re.hypot(im)>1e-12 { Some(im.atan2(re)) } else { None }
        }
        7 => {
            let sin_re=x.sin()*y.cosh();
            let sin_im=x.cos()*y.sinh();
            let cos_re=x.cos()*y.cosh();
            let cos_im= -x.sin()*y.sinh();
            let denominator=cos_re*cos_re+cos_im*cos_im;
            let re=sin_re*cos_re+sin_im*cos_im;
            let im=sin_im*cos_re-sin_re*cos_im;
            if denominator>1e-12 && re.hypot(im)>1e-12 { Some(im.atan2(re)) } else { None }
        }
        _ => None,
    }
}

fn color_for_angle(angle: f64) -> Srgb {
    let angle=angle.rem_euclid(std::f64::consts::TAU);
    let index=((angle/std::f64::consts::TAU*COLORS.len() as f64).floor() as usize).min(COLORS.len()-1);
    hex(COLORS[index])
}



impl SceneConstruct for DomainColor {
    fn name(&self) -> &str { "01_domain_color" }

    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit=self.kit.clone();
        kit.say(stage,"01_1")?;
        let title=kit.text(stage,"Color the direction of each output",34.0,Palette::ink(),v(0.0,2.9))?;
        let legend=kit.text(stage,"Hue = output angle; brightness here is fixed",23.0,Palette::ink(),v(2.4,1.8))?;
        let explanation=kit.text(stage,"Each tile samples an input point",22.0,Palette::dim(),v(2.4,1.1))?;
        let legend_start=-0.65_f64;
        let mut permanent=vec![title,legend,explanation];
        for (i,color) in COLORS.iter().enumerate() {
            let x=0.1+i as f64*0.43;
            permanent.push(stage.arena_mut().add(rect(v(x,legend_start),0.42,0.32,ink_style(hex(color)))?));
        }
        permanent.push(kit.text(stage,"0                          one turn",20.0,Palette::dim(),v(2.45,-1.05))?);
        fade(stage,&permanent,true)?;
        let names=["f(z) = z²", "f(z) = 1/z; zero excluded", "f(z) = exp(z)", "f(z) = z³ − 1", "f(z) = sin(z)", "f(z) = (z + 1)/(z − 1)", "f(z) = cos(z)", "f(z) = tan(z)"];
        // Preserve the original sampled domain and screen footprint while
        // providing enough cells to make the changing phase easier to see.
        let n=31_i32;
        let half=(n-1)/2;
        let tile=4.05_f64/f64::from(n);
        let spacing=1.54_f64/f64::from(half);
        for (kind,name) in names.iter().enumerate() {
            if kind==3 { kit.say(stage,"01_2")?; }
            if kind==5 { kit.say(stage,"01_3")?; }
            let label=kit.text(stage,name,27.0,Palette::hero(),v(-3.1,-2.5))?;
            let mut grid=vec![label];
            if kind>=6 {
                grid.push(kit.text(stage,"horizontal span ≈ 2π",20.0,Palette::ink(),v(-3.1,-2.95))?);
            }
            for row in 0..n {
                for col in 0..n {
                    // Show slightly more than one cosine period for the trigonometric examples.
                    let x_step=if kind>=6 { 3.2/f64::from(half) } else { spacing };
                    let x=f64::from(col-half)*x_step;
                    let y=f64::from(row-half)*spacing;
                    let color=phase(kind as u8,x,y).map(color_for_angle).unwrap_or(Palette::faint());
                    grid.push(stage.arena_mut().add(rect(v(-5.125+f64::from(col)*tile,-2.0+f64::from(row)*tile),tile,tile,ink_style(color))?));
                }
            }
            if kind==3 {
                // Exact zeros are overlaid on the sampled phase tiles; the
                // roots themselves need not be tile centers.
                let root_y=3.0_f64.sqrt()/2.0;
                for (x,y) in [(1.0,0.0),(-0.5,root_y),(-0.5,-root_y)] {
                    let cx=-5.125+(f64::from(half)+x/spacing)*tile+tile/2.0;
                    let cy=-2.0+(f64::from(half)+y/spacing)*tile+tile/2.0;
                    let dot=Circle::new().radius(0.095).style(ink_style(Palette::ink())).build().moved_to(v(cx,cy));
                    grid.push(stage.arena_mut().add(dot));
                }
                grid.push(kit.text(stage,"white dots: exact zeros",21.0,Palette::ink(),v(-3.1,-2.9))?);
            }
            if kind==5 {
                // Mark the exact zero and pole even when neither is a tile center.
                for (x,color) in [(-1.0,Palette::ink()),(1.0,Palette::hero())] {
                    let cx=-5.125+(f64::from(half)+x/spacing)*tile+tile/2.0;
                    let cy=-2.0+f64::from(half)*tile+tile/2.0;
                    let backing=Circle::new().radius(0.15).style(ink_style(hex("#111620"))).build().moved_to(v(cx,cy));
                    grid.push(stage.arena_mut().add(backing));
                    let dot=Circle::new().radius(0.09).style(ink_style(color)).build().moved_to(v(cx,cy));
                    grid.push(stage.arena_mut().add(dot));
                }
                grid.push(kit.text(stage,"white: zero at z = −1",24.0,Palette::ink(),v(2.4,-1.75))?);
                grid.push(kit.text(stage,"gold: pole at z = 1",24.0,Palette::hero(),v(2.4,-2.35))?);
            }
            fade(stage,&grid,true)?;
            stage.wait(2.0)?;
            // Scratch sentence starts are estimates; real narration needs verified boundaries.
            if kind==0 { kit.until_sentence(stage,"01_1",2)?; }
            if kind==1 { kit.until_sentence(stage,"01_1",3)?; }
            // Keep the exponential field visible until the first cue ends;
            // the supplemental cue starts only after this hold.
            if kind==2 { kit.hold(stage)?; }
            // Wait for the second sentence (Sine) in cue 01_2; scratch timings are estimates.
            if kind==3 { kit.until_sentence(stage,"01_2",1)?; }
            if kind==4 { kit.hold(stage)?; }
            if kind==5 { kit.until_sentence(stage,"01_3",1)?; }
            if kind==6 { kit.until_sentence(stage,"01_3",2)?; }
            if kind+1<names.len() { fade(stage,&grid,false)?; }
        }
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::phase;
    #[test]
    fn example_phases_and_origin() {
        assert!(phase(0,0.0,0.0).is_none());
        assert!(phase(1,0.0,0.0).is_none());
        assert!((phase(0,0.8,0.6).unwrap()-0.96_f64.atan2(0.28)).abs()<1e-12);
        assert!((phase(1,0.0,1.0).unwrap()+std::f64::consts::FRAC_PI_2).abs()<1e-12);
        assert!((phase(2,3.0,0.7).unwrap()-0.7).abs()<1e-12);
        assert!(phase(3,1.0,0.0).is_none());
        assert!((phase(3,0.0,0.0).unwrap()-std::f64::consts::PI).abs()<1e-12);
        assert!(phase(4,0.0,0.0).is_none());
        assert!((phase(4,0.0,1.0).unwrap()-std::f64::consts::FRAC_PI_2).abs()<1e-12);
        assert!(phase(4,std::f64::consts::FRAC_PI_2,0.0).unwrap().abs()<1e-12);
        assert!(phase(5,1.0,0.0).is_none());
        assert!(phase(5,-1.0,0.0).is_none());
        assert!((phase(5,0.0,1.0).unwrap()+std::f64::consts::FRAC_PI_2).abs()<1e-12);
        assert!(phase(6,0.0,0.0).unwrap().abs()<1e-12);
        assert!(phase(7,0.0,0.0).is_none());
        assert!((phase(7,0.0,1.0).unwrap()-std::f64::consts::FRAC_PI_2).abs()<1e-12);
    }
}
