//! Dimension ladder and generalized theorem statement; unbuilt scratch draft.
use explainer_kit::{Kit,Palette,ink_style,v};
use fmn::prelude::*;
use super::opening::{arrow,fade,line};
pub struct Dimensions {pub kit:Kit}
impl SceneConstruct for Dimensions {
 fn name(&self)->&str {"08_dimensions"}
 fn construct(&mut self,stage:&mut Stage<'_>)->fmn::Result<()> {
  let kit=self.kit.clone();let title=kit.text(stage,"Dimension tells us what to integrate",34.0,Palette::ink(),v(0.0,3.0))?;
  let names=[("0-form",-5.1),("1-form",-1.7),("2-form",1.7),("3-form",5.1)];let mut diagram=vec![title];for (name,x) in names {diagram.push(kit.text(stage,name,26.0,Palette::hero(),v(x,2.0))?);}
  let p=stage.arena_mut().add(Circle::new().radius(0.12).style(ink_style(Palette::blue())).build().moved_to(v(-5.1,0.4)));diagram.push(p);
  diagram.extend(arrow(stage,(-2.6,0.4),(-0.8,0.4),Palette::blue())?);
  for (a,b) in [((0.8,-0.4),(2.6,-0.4)),((2.6,-0.4),(2.6,1.2)),((2.6,1.2),(0.8,1.2)),((0.8,1.2),(0.8,-0.4))] {diagram.extend(arrow(stage,a,b,Palette::blue())?);}
  for (a,b) in [((4.2,-0.4),(5.7,-0.4)),((5.7,-0.4),(5.7,0.9)),((5.7,0.9),(4.2,0.9)),((4.2,0.9),(4.2,-0.4)),((4.2,0.9),(4.6,1.3)),((5.7,0.9),(6.1,1.3)),((4.6,1.3),(6.1,1.3)),((6.1,1.3),(6.1,0.0)),((6.1,0.0),(5.7,-0.4))] {diagram.push(line(stage,a,b,Palette::blue(),2.5)?);}
  let labels=[("points",-5.1),("oriented steps",-1.7),("oriented area",1.7),("oriented volume",5.1)];for (name,x) in labels {diagram.push(kit.text(stage,name,22.0,Palette::ink(),v(x,-1.15))?);}
  fade(stage,&diagram,true)?;kit.say(stage,"08_1")?;
  kit.until_sentence(stage,"08_1",2)?;let rule=kit.text(stage,"k-form: alternating multilinear rule on k directions",25.0,Palette::ink(),v(0.0,-1.9))?;fade(stage,&[rule],true)?;
  kit.until_sentence(stage,"08_1",3)?;let sign=kit.text(stage,"swap two directions → change sign",25.0,Palette::hero(),v(0.0,-2.55))?;fade(stage,&[sign],true)?;
  kit.until_sentence(stage,"08_1",4)?;let ext=kit.text(stage,"d maps k-forms to (k+1)-forms",24.0,Palette::green(),v(0.0,-3.1))?;fade(stage,&[ext],true)?;kit.hold(stage)?;
  let mut remove=diagram;remove.extend([rule,sign,ext]);fade(stage,&remove,false)?;
  let heading=kit.text(stage,"Generalized Stokes",34.0,Palette::ink(),v(0.0,2.8))?;
  let equation=kit.text(stage,"∫ on M dω   =   ∫ on ∂M ω",38.0,Palette::hero(),v(0.0,1.2))?;
  let domain=kit.text(stage,"M: compact, oriented, smooth n-manifold with boundary",25.0,Palette::ink(),v(0.0,0.0))?;
  let degree=kit.text(stage,"ω: smooth (n−1)-form; boundary has induced orientation",25.0,Palette::ink(),v(0.0,-0.8))?;
  let path=kit.text(stage,"Proof: oriented box → curved manifold",26.0,Palette::green(),v(0.0,-1.8))?;
  kit.say(stage,"08_2")?;fade(stage,&[heading,equation],true)?;fade(stage,&[domain,degree],true)?;kit.until_sentence(stage,"08_2",2)?;fade(stage,&[path],true)?;kit.finish(stage)
 }
}
