//! Assemble chartwise identities with a partition of unity; draft, not reviewed.
use explainer_kit::{Kit,Palette,v};
use fmn::prelude::*;
use super::opening::{fade,line};
pub struct Assemble {pub kit:Kit}
impl SceneConstruct for Assemble {
 fn name(&self)->&str {"13_assemble"}
 fn construct(&mut self,stage:&mut Stage<'_>)->fmn::Result<()> {
  let kit=self.kit.clone();let title=kit.text(stage,"Local pieces fit back together",34.0,Palette::ink(),v(0.0,3.0))?;
  let pieces=kit.text(stage,"rho_1 ω  +  rho_2 ω  +  ···  =  ω",29.0,Palette::hero(),v(2.6,1.8))?; let axes=vec![line(stage,(-5.3,-1.2),(-1.3,-1.2),Palette::dim(),2.0)?,line(stage,(-5.3,-1.2),(-5.3,1.4),Palette::dim(),2.0)?];let weights=vec![line(stage,(-5.3,-1.2),(-1.5,1.2),Palette::blue(),3.0)?,line(stage,(-5.3,1.2),(-1.5,-1.2),Palette::green(),3.0)?];let toy=kit.text(stage,"example: weights add to 1",21.0,Palette::ink(),v(-3.3,-1.9))?;let one=kit.text(stage,"rho_1",20.0,Palette::green(),v(-5.0,1.6))?;let two=kit.text(stage,"rho_2",20.0,Palette::blue(),v(-1.7,1.6))?;fade(stage,&[title,pieces,toy,one,two],true)?;fade(stage,&axes,true)?;fade(stage,&weights,true)?;kit.say(stage,"13_1")?;
  kit.until_sentence(stage,"13_1",2)?;let product=kit.text(stage,"d(rho_i ω) = d(rho_i) ∧ ω + rho_i dω",27.0,Palette::ink(),v(2.6,0.7))?;fade(stage,&[product],true)?;
  kit.until_sentence(stage,"13_1",3)?;let zero=kit.text(stage,"Σ d(rho_i) = d(Σ rho_i) = d(1) = 0",27.0,Palette::green(),v(2.6,-0.35))?;fade(stage,&[zero],true)?;
  kit.until_sentence(stage,"13_1",4)?;let cancel=kit.text(stage,"Extra weight-derivative terms cancel",25.0,Palette::green(),v(2.6,-1.35))?;fade(stage,&[cancel],true)?;kit.hold(stage)?;
  kit.say(stage,"13_2")?;fade(stage,&[pieces,product,zero,cancel,toy,one,two],false)?;fade(stage,&weights,false)?;fade(stage,&axes,false)?;let a=kit.text(stage,"Pullback commutes with d",27.0,Palette::hero(),v(0.0,1.3))?;let b=kit.text(stage,"Oriented change of coordinates",27.0,Palette::ink(),v(0.0,0.4))?;let c=kit.text(stage,"Smooth partitions of unity exist",27.0,Palette::ink(),v(0.0,-0.5))?;let caveat=kit.text(stage,"These are ingredients, not proved by the picture",23.0,Palette::dim(),v(0.0,-1.6))?;fade(stage,&[a,b,c],true)?;kit.until_sentence(stage,"13_2",1)?;fade(stage,&[caveat],true)?;kit.finish(stage)
 }
}
