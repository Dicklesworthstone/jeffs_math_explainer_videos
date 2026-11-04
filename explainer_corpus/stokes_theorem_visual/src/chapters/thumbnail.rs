//! Native draft thumbnail: adjacent oriented tiles and their cancelling seam.
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;
use super::opening::line;

pub fn make(kit: Kit) -> Box<dyn SceneConstruct> { Box::new(Thumbnail { kit }) }
struct Thumbnail { kit: Kit }

// Larger strokes and arrowheads for a reduced-size thumbnail.
fn arrow(stage: &mut Stage<'_>, a: (f64, f64), b: (f64, f64), color: Srgb) -> fmn::Result<Vec<Mob>> {
    let (dx,dy)=(b.0-a.0,b.1-a.1); let len=dx.hypot(dy);
    if len <= f64::EPSILON { return Err(fmn::Error::Scene(fmn::scene::SceneError::InvalidConfig("zero-length boundary edge"))); }
    let (ux,uy)=(dx/len,dy/len); let tip=(a.0+0.58*dx,a.1+0.58*dy);
    let base=(tip.0-0.36*ux,tip.1-0.36*uy); let wing=0.20;
    Ok(vec![line(stage,a,b,color,6.0)?,line(stage,tip,(base.0-wing*uy,base.1+wing*ux),color,6.0)?,line(stage,tip,(base.0+wing*uy,base.1-wing*ux),color,6.0)?])
}

impl SceneConstruct for Thumbnail {
    fn name(&self) -> &str { "thumbnail" }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        for (text,size,color,point) in [
            ("THE MAGIC OF", 55.0, Palette::ink(), v(-3.75, 2.55)),
            ("STOKES'", 79.0, Palette::gold(), v(-3.75, 1.15)),
            ("THEOREM", 75.0, Palette::ink(), v(-3.75, -0.1)),
            ("INSIDE AND BOUNDARY", 30.0, Palette::blue(), v(-3.75, -1.65)),
            ("SHARED EDGES CANCEL", 29.0, Palette::ink(), v(3.7, -2.2)),
        ] { let mob=kit.text_styled(stage,text,size,color,point,true,false)?; stage.add_to_scene(mob)?; }
        // Two counterclockwise oriented squares: their common edge is traversed
        // in opposite directions. Small offsets reveal both directed copies.
        for (a,b) in [((1.2,-0.9),(3.7,-0.9)),((3.7,-0.9),(6.2,-0.9)),((6.2,-0.9),(6.2,1.6)),((6.2,1.6),(3.7,1.6)),((3.7,1.6),(1.2,1.6)),((1.2,1.6),(1.2,-0.9))] {
            for mob in arrow(stage,a,b,Palette::blue())? { stage.add_to_scene(mob)?; }
        }
        for mob in arrow(stage,(3.56,-0.9),(3.56,1.6),Palette::hero())? { stage.add_to_scene(mob)?; }
        for mob in arrow(stage,(3.84,1.6),(3.84,-0.9),Palette::green())? { stage.add_to_scene(mob)?; }
        stage.wait(1.0)?; Ok(())
    }
}
