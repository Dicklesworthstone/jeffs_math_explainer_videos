//! The inner boundary of an annulus; unbuilt and unreviewed draft.
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;
use super::opening::{arrow,fade,line};

pub struct Annulus { pub kit:Kit }
fn pt(r:f64,t:f64)->(f64,f64){(-2.4+r*t.cos(),0.1+r*t.sin())}
fn ring(stage:&mut Stage<'_>,r:f64,color:Srgb)->fmn::Result<Vec<Mob>>{
 let mut v=Vec::new();let tau=std::f64::consts::TAU;
 for i in 0..48 {let a=tau*(i as f64)/48.0;let b=tau*((i+1) as f64)/48.0;v.push(line(stage,pt(r,a),pt(r,b),color,2.5)?);}Ok(v)
}
impl SceneConstruct for Annulus {
 fn name(&self)->&str {"07_annulus"}
 fn construct(&mut self,stage:&mut Stage<'_>)->fmn::Result<()> {
  let kit=self.kit.clone();let tau=std::f64::consts::TAU;
  let title=kit.text(stage,"A boundary you might forget",34.0,Palette::ink(),v(0.0,3.1))?;
  let mut region=ring(stage,2.0,Palette::blue())?;region.extend(ring(stage,1.0,Palette::blue())?);
  for i in 0..24 {let t=tau*(i as f64)/24.0;region.push(line(stage,pt(1.0,t),pt(2.0,t),Palette::faint(),1.2)?);}
  let hole=kit.text(stage,"origin excluded",22.0,Palette::dim(),v(-2.4,0.1))?;
  let question=kit.text(stage,"Is the outer edge the whole boundary?",25.0,Palette::hero(),v(2.6,1.8))?;
  let question2=kit.text(stage,"Which way does each edge go?",25.0,Palette::ink(),v(2.6,1.0))?;
  let mut initial=vec![title,hole,question,question2];initial.extend(region.iter().copied());fade(stage,&initial,true)?;
  kit.say(stage,"07_1")?;kit.hold(stage)?;kit.mark(stage,"predict");stage.wait(3.0)?;
  kit.say(stage,"07_2")?;fade(stage,&[question,question2],false)?;
  let mut orient=Vec::new();for t in [0.4,2.5,4.6] {orient.extend(arrow(stage,pt(2.0,t),pt(2.0,t+0.34),Palette::hero())?);orient.extend(arrow(stage,pt(1.0,t+0.34),pt(1.0,t),Palette::green())?);}fade(stage,&orient,true)?;
  let outer=kit.text(stage,"outer: counterclockwise",24.0,Palette::hero(),v(2.7,1.9))?;let inner=kit.text(stage,"inner: clockwise",24.0,Palette::green(),v(2.7,1.25))?;fade(stage,&[outer,inner],true)?;
  kit.until_sentence(stage,"07_2",1)?;let omega=kit.text(stage,"ω = (−y dx + x dy)/(x² + y²)",23.0,Palette::ink(),v(2.7,0.45))?;fade(stage,&[omega],true)?;
  kit.until_sentence(stage,"07_2",2)?;let smooth=kit.text(stage,"smooth here; dω = 0",23.0,Palette::blue(),v(2.7,-0.2))?;fade(stage,&[smooth],true)?;
  kit.until_sentence(stage,"07_2",3)?;let sums=kit.text(stage,"outer: 2π  +  inner: −2π  = 0",24.0,Palette::green(),v(2.7,-1.05))?;fade(stage,&[sums],true)?;kit.hold(stage)?;
  kit.say(stage,"07_3")?;let warning=kit.text(stage,"Cannot fill the hole: ω undefined at 0",24.0,Palette::red(),v(0.0,-2.8))?;fade(stage,&[warning],true)?;
  kit.finish(stage)
 }
}
