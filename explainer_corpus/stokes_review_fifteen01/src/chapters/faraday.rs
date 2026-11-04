//! Fixed-loop Maxwell–Faraday example; draft, not built or reviewed.
use explainer_kit::{Kit,Palette,v};
use fmn::prelude::*;
use super::opening::{arrow,fade,line};
pub struct Faraday {pub kit:Kit}
impl SceneConstruct for Faraday {
 fn name(&self)->&str {"14_faraday"}
 fn construct(&mut self,stage:&mut Stage<'_>)->fmn::Result<()> {
  let kit=self.kit.clone(); let title=kit.text(stage,"A fixed loop in a changing field",34.0,Palette::ink(),v(0.0,3.0))?;let fixed=kit.text(stage,"loop and surface fixed",22.0,Palette::dim(),v(-3.3,-2.3))?;
  let mut rim=Vec::new();let tau=std::f64::consts::TAU;let point=|t:f64|(-3.3+2.0*t.cos(),-0.2+0.8*t.sin());for i in 0..48 {let a=tau*(i as f64)/48.0;let b=tau*((i+1) as f64)/48.0;rim.push(line(stage,point(a),point(b),Palette::blue(),2.5)?);}let mut initial=vec![title,fixed];initial.extend(rim);fade(stage,&initial,true)?;kit.say(stage,"14_1")?;
  kit.until_sentence(stage,"14_1",1)?;let mut field_short=Vec::new();let mut field_long=Vec::new();for x in [-4.3,-3.3,-2.3] {field_short.extend(arrow(stage,(x,0.15),(x,0.7),Palette::green())?);field_long.extend(arrow(stage,(x,0.15),(x,1.35),Palette::green())?);}fade(stage,&field_short,true)?;let changing=kit.text(stage,"B increases upward",23.0,Palette::green(),v(-3.3,1.9))?;fade(stage,&[changing],true)?;stage.wait(0.8)?;fade(stage,&field_short,false)?;fade(stage,&field_long,true)?;
  kit.until_sentence(stage,"14_1",2)?;let mut circulation=Vec::new();for t in [0.1_f64,1.7,3.3,4.9] {circulation.extend(arrow(stage,point(t),point(t-0.35),Palette::hero())?);}fade(stage,&circulation,true)?;let direction=kit.text(stage,"E circulates clockwise",21.0,Palette::hero(),v(-3.3,-1.65))?;fade(stage,&[direction],true)?;let law=kit.text(stage,"∮ E · dl = − d/dt ∫ B · n dA",27.0,Palette::hero(),v(2.5,1.0))?;fade(stage,&[law],true)?;kit.until_sentence(stage,"14_1",3)?;let note=kit.text(stage,"minus sign comes from Faraday's law",21.0,Palette::ink(),v(2.5,-0.1))?;fade(stage,&[note],true)?;kit.hold(stage)?;
  kit.say(stage,"14_2")?;let curl=kit.text(stage,"∫ curl E · n dA = −∫ ∂B/∂t · n dA",25.0,Palette::ink(),v(2.5,-0.95))?;fade(stage,&[curl],true)?;kit.until_sentence(stage,"14_2",2)?;let local=kit.text(stage,"curl E = −∂B/∂t",29.0,Palette::hero(),v(2.5,-1.8))?;fade(stage,&[local],true)?;kit.until_sentence(stage,"14_2",4)?;let caveat=kit.text(stage,"moving loops need extra care",21.0,Palette::dim(),v(0.0,-2.8))?;fade(stage,&[caveat],true)?;kit.finish(stage)
 }
}
