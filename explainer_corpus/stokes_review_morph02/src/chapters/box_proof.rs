//! Coordinate-slice proof on an oriented box; unbuilt and unreviewed draft.
use explainer_kit::{Kit,Palette,v};
use fmn::prelude::*;
use super::opening::{arrow,fade,line};
pub struct BoxProof {pub kit:Kit}
impl SceneConstruct for BoxProof {
 fn name(&self)->&str {"11_box_proof"}
 fn construct(&mut self,stage:&mut Stage<'_>)->fmn::Result<()> {
  let kit=self.kit.clone();let title=kit.text(stage,"One coordinate at a time",34.0,Palette::ink(),v(0.0,3.0))?;
  let mut boxlines=Vec::new();for (a,b) in [((-5.1,-1.0),(-1.0,-1.0)),((-1.0,-1.0),(-1.0,1.6)),((-1.0,1.6),(-5.1,1.6)),((-5.1,1.6),(-5.1,-1.0))] {boxlines.push(line(stage,a,b,Palette::blue(),2.5)?);}
  let a=kit.text(stage,"a_i",25.0,Palette::ink(),v(-5.1,-1.5))?;let b=kit.text(stage,"b_i",25.0,Palette::ink(),v(-1.0,-1.5))?;
  let axis=kit.text(stage,"i-th coordinate →",23.0,Palette::dim(),v(-3.0,-2.2))?;
  let mut first=vec![title,a,b,axis];first.extend(boxlines);fade(stage,&first,true)?;
  kit.say(stage,"11_1")?;let mut slice=arrow(stage,(-5.1,0.3),(-1.0,0.3),Palette::hero())?;fade(stage,&slice,true)?;
  kit.until_sentence(stage,"11_1",1)?;let ftc=kit.text(stage,"∫ from a_i to b_i ∂_if_i dx^i = f_i(b_i) − f_i(a_i)",27.0,Palette::ink(),v(3.0,1.5))?;fade(stage,&[ftc],true)?;
  kit.until_sentence(stage,"11_1",2)?;for y in [-0.7_f64,-0.2,0.8,1.3] {let s=arrow(stage,(-5.1,y),(-1.0,y),Palette::hero())?;fade(stage,&s,true)?;slice.extend(s);}
  let sweep=kit.text(stage,"Integrate over the other coordinates",24.0,Palette::green(),v(2.8,0.5))?;fade(stage,&[sweep],true)?;
  kit.until_sentence(stage,"11_1",3)?;let fubini=kit.text(stage,"Smooth on a bounded box: use Fubini",23.0,Palette::dim(),v(2.8,-0.4))?;fade(stage,&[fubini],true)?;kit.hold(stage)?;
  kit.say(stage,"11_2")?;let signs=kit.text(stage,"Upper face +; lower face −",27.0,Palette::hero(),v(2.8,-1.35))?;fade(stage,&[signs],true)?;
  kit.until_sentence(stage,"11_2",2)?;let sum=kit.text(stage,"Σ face pairs = ∫ on ∂box ω",27.0,Palette::green(),v(0.0,-2.8))?;fade(stage,&[sum],true)?;
  kit.until_sentence(stage,"11_2",3)?;let proof=kit.text(stage,"FTC + Fubini + induced orientations",23.0,Palette::ink(),v(2.8,-2.15))?;fade(stage,&[proof],true)?;
  kit.finish(stage)
 }
}
