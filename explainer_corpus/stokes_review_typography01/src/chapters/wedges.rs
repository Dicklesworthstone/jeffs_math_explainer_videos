//! Repeated differentials and the surviving term; unbuilt scratch draft.
use explainer_kit::{Kit,Palette,v};
use fmn::prelude::*;
use super::opening::fade;
pub struct Wedges {pub kit:Kit}
impl SceneConstruct for Wedges {
 fn name(&self)->&str {"10_wedge_cancellation"}
 fn construct(&mut self,stage:&mut Stage<'_>)->fmn::Result<()> {
  let kit=self.kit.clone();let title=kit.text(stage,"Why only one derivative survives",34.0,Palette::ink(),v(0.0,3.0))?;
  let term=kit.text(stage,"Example term: −f₂ dx ∧ dz",30.0,Palette::hero(),v(0.0,2.0))?;
  let repeated=kit.text(stage,"dx ∧ dx = 0      dz ∧ dz = 0",30.0,Palette::red(),v(0.0,0.9))?;
  fade(stage,&[title,term,repeated],true)?;kit.say(stage,"10_1")?;
  kit.until_sentence(stage,"10_1",2)?;let surviving=kit.text(stage,"Only dy is missing; ∂ᵧf₂ dy survives",27.0,Palette::blue(),v(0.0,-0.1))?;fade(stage,&[surviving],true)?;
  kit.until_sentence(stage,"10_1",3)?;let signs=kit.text(stage,"−∂ᵧf₂ dy ∧ dx ∧ dz  =  +∂ᵧf₂ dx ∧ dy ∧ dz",27.0,Palette::green(),v(0.0,-1.1))?;let note=kit.text(stage,"One swap cancels the earlier minus",24.0,Palette::ink(),v(0.0,-2.0))?;fade(stage,&[signs,note],true)?;kit.hold(stage)?;
  kit.say(stage,"10_2")?;fade(stage,&[term,repeated,surviving,signs,note],false)?;
  let general=kit.text(stage,"dω = (∑ᵢ ∂ᵢfᵢ) dx¹ ∧ ··· ∧ dxⁿ",32.0,Palette::hero(),v(0.0,1.2))?;
  let analogy=kit.text(stage,"In 3D the coefficient is divergence",27.0,Palette::ink(),v(0.0,0.2))?;
  let next=kit.text(stage,"Next: integrate one coordinate at a time",26.0,Palette::green(),v(0.0,-1.0))?;
  fade(stage,&[general],true)?;kit.until_sentence(stage,"10_2",1)?;fade(stage,&[analogy],true)?;kit.until_sentence(stage,"10_2",2)?;fade(stage,&[next],true)?;kit.finish(stage)
 }
}
