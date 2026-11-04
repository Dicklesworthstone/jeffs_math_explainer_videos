//! First two Stokes scenes. Draft source; narration, timing and review remain open.
use explainer_kit::{Kit, Palette, Timed, stroke_style, v};
use fmn::prelude::*;

pub struct Seams { pub kit: Kit }
pub struct Interval { pub kit: Kit }

const LEFT: f64 = -2.7;
const SEAM: f64 = -0.2;
const RIGHT: f64 = 2.3;
const BOTTOM: f64 = -1.0;
const TOP: f64 = 1.5;

pub(super) fn fade(stage: &mut Stage<'_>, mobs: &[Mob], incoming: bool) -> fmn::Result<()> {
    let mut animations: Vec<Box<dyn Animation>> = Vec::new();
    for &mob in mobs {
        let animation = if incoming { fade_in(stage.arena_mut(),mob,ORIGIN,1.0)? }
            else { fade_out(stage.arena_mut(),mob,ORIGIN,1.0)? };
        animations.push(stage.prepare(animation.rt(0.5))?);
    }
    stage.play_prepared(animations)?;
    Ok(())
}

pub(super) fn line(stage: &mut Stage<'_>, a: (f64,f64), b: (f64,f64), color: Srgb, width: f64) -> fmn::Result<Mob> {
    Ok(stage.arena_mut().add(Line::new(v(a.0,a.1),v(b.0,b.1)).style(stroke_style(color,width)).build()?))
}

/// Directed edge with a midpoint arrowhead; geometry is generated from endpoints.
pub(super) fn arrow(stage: &mut Stage<'_>, a: (f64,f64), b: (f64,f64), color: Srgb) -> fmn::Result<Vec<Mob>> {
    let (dx,dy)=(b.0-a.0,b.1-a.1);let len=dx.hypot(dy);
    if len <= f64::EPSILON { return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("zero-length boundary edge"))); }
    let (ux,uy)=(dx/len,dy/len);let tip=(a.0+0.58*dx,a.1+0.58*dy);let base=(tip.0-0.23*ux,tip.1-0.23*uy);let wing=0.13;
    Ok(vec![line(stage,a,b,color,3.0)?,line(stage,tip,(base.0-wing*uy,base.1+wing*ux),color,3.0)?,line(stage,tip,(base.0+wing*uy,base.1-wing*ux),color,3.0)?])
}

impl SceneConstruct for Seams {
    fn name(&self)->&str { "01_disappearing_seams" }
    fn construct(&mut self,stage:&mut Stage<'_>)->fmn::Result<()> {
        let kit=self.kit.clone();
        let title=kit.text(stage,"What survives when boundaries add?",34.0,Palette::ink(),v(0.0,3.0))?;
        let note=kit.text(stage,"Each square is traversed counterclockwise",25.0,Palette::dim(),v(0.0,2.25))?;
        let question=kit.text(stage,"Which edges survive?",29.0,Palette::hero(),v(0.0,-2.45))?;
        let mut outer=Vec::new();
        // Six exterior directed edges of the two counterclockwise squares.
        for (a,b) in [((LEFT,BOTTOM),(SEAM,BOTTOM)),((SEAM,BOTTOM),(RIGHT,BOTTOM)),((RIGHT,BOTTOM),(RIGHT,TOP)),((RIGHT,TOP),(SEAM,TOP)),((SEAM,TOP),(LEFT,TOP)),((LEFT,TOP),(LEFT,BOTTOM))] {
            outer.extend(arrow(stage,a,b,Palette::blue())?);
        }
        // The offset strokes are the same shared edge traversed in opposite directions.
        let mut shared=arrow(stage,(SEAM-0.09,BOTTOM),(SEAM-0.09,TOP),Palette::hero())?;
        shared.extend(arrow(stage,(SEAM+0.09,TOP),(SEAM+0.09,BOTTOM),Palette::green())?);
        let shared_label=kit.text(stage,"Shared edge: opposite directions",24.0,Palette::ink(),v(0.0,-1.7))?;
        let mut visible=vec![title,note,question,shared_label];visible.extend(outer);visible.extend(shared.iter().copied());fade(stage,&visible,true)?;
        kit.say(stage,"01_1")?;kit.hold(stage)?;kit.mark(stage,"predict");stage.wait(3.0)?;
        kit.say(stage,"01_2")?;fade(stage,&shared,false)?;fade(stage,&[shared_label,question],false)?;
        let cancel=kit.text(stage,"Interior seam cancels",30.0,Palette::green(),v(0.0,-1.65))?;
        let result=kit.text(stage,"Only the outside boundary remains",28.0,Palette::hero(),v(0.0,-2.45))?;
        fade(stage,&[cancel,result],true)?;
        kit.finish(stage)
    }
}

impl SceneConstruct for Interval {
    fn name(&self)->&str { "02_interval" }
    fn construct(&mut self,stage:&mut Stage<'_>)->fmn::Result<()> {
        let kit=self.kit.clone();
        let title=kit.text(stage,"An interval has a signed boundary",34.0,Palette::ink(),v(0.0,3.0))?;
        let axis=line(stage,(-4.3,0.6),(4.3,0.6),Palette::blue(),4.0)?;
        let mut marks=Vec::new();
        for x in [-4.3_f64,-1.45,1.45,4.3] { marks.push(line(stage,(x,0.42),(x,0.78),Palette::ink(),3.0)?); }
        let mut labels=Vec::new();
        for (name,x) in [("a",-4.3),("x₁",-1.45),("x₂",1.45),("b",4.3)] { labels.push(kit.text(stage,name,27.0,Palette::ink(),v(x,1.15))?); }
        let ends=[kit.text(stage,"−",36.0,Palette::hero(),v(-4.3,-0.05))?,kit.text(stage,"+",36.0,Palette::green(),v(4.3,-0.05))?];
        let inside=[kit.text(stage,"+   −",28.0,Palette::hero(),v(-1.45,-0.05))?,kit.text(stage,"+   −",28.0,Palette::hero(),v(1.45,-0.05))?];
        let boundary=kit.text(stage,"Boundary [a,b] = b − a",29.0,Palette::hero(),v(0.0,-1.1))?;
        let mut visible=vec![title,axis,boundary];visible.extend(marks);visible.extend(labels);visible.extend(ends);visible.extend(inside);fade(stage,&visible,true)?;
        kit.say(stage,"02_1")?;kit.until_sentence(stage,"02_1",3)?;fade(stage,&inside,false)?;
        let cancel=kit.text(stage,"Interior endpoints cancel",26.0,Palette::green(),v(0.0,-1.75))?;fade(stage,&[cancel],true)?;kit.hold(stage)?;
        kit.say(stage,"02_2")?;kit.until_sentence(stage,"02_2",2)?;
        let ftc=kit.text(stage,"∫ from a to b  f′(x) dx  =  f(b) − f(a)",28.0,Palette::ink(),v(0.0,-2.55))?;
        fade(stage,&[ftc],true)?;
        kit.finish(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::{BOTTOM,LEFT,RIGHT,SEAM,TOP};
    #[test]
    fn displayed_tiles_have_equal_width_and_height() {
        assert!(((SEAM-LEFT)-(TOP-BOTTOM)).abs()<1e-12);
        assert!(((RIGHT-SEAM)-(TOP-BOTTOM)).abs()<1e-12);
    }
    #[test]
    fn two_neighboring_square_boundaries_cancel_one_shared_edge() {
        let left=[((0,0),(1,0)),((1,0),(1,1)),((1,1),(0,1)),((0,1),(0,0))];
        let right=[((1,0),(2,0)),((2,0),(2,1)),((2,1),(1,1)),((1,1),(1,0))];
        assert!(left.iter().any(|&(a,b)|right.contains(&(b,a))));
        assert_eq!(left.len()+right.len()-2,6);
    }
}
