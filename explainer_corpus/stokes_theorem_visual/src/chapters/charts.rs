//! Coordinate charts and a partition of unity; draft, not built or reviewed.
use explainer_kit::{Kit,Palette,v};
use fmn::prelude::*;
use super::opening::{fade,line};
pub struct Charts {pub kit:Kit}
impl SceneConstruct for Charts {
 fn name(&self)->&str {"12_charts"}
 fn construct(&mut self,stage:&mut Stage<'_>)->fmn::Result<()> {
  let kit=self.kit.clone(); let title=kit.text(stage,"From a box to a curved manifold",34.0,Palette::ink(),v(0.0,3.0))?;
  let mut boundary=Vec::new(); let points=[(-5.8,-1.5),(-4.5,-1.8),(-3.0,-1.6),(-1.6,-1.0),(-0.7,-0.2)]; for pair in points.windows(2) { boundary.push(line(stage,pair[0],pair[1],Palette::blue(),3.0)?); }
  let mut chart_a=Vec::new(); for (a,b) in [((-5.3,-0.8),(-2.8,-0.8)),((-2.8,-0.8),(-2.8,1.2)),((-2.8,1.2),(-5.3,1.2)),((-5.3,1.2),(-5.3,-0.8))] {chart_a.push(line(stage,a,b,Palette::hero(),2.2)?);}
  let mut chart_b=Vec::new(); for (a,b) in [((-3.6,-2.0),(-1.0,-2.0)),((-1.0,-2.0),(-1.0,0.3)),((-1.0,0.3),(-3.6,0.3)),((-3.6,0.3),(-3.6,-2.0))] {chart_b.push(line(stage,a,b,Palette::green(),2.2)?);}
  let mut inside=Vec::new(); for (x,y) in [(-3.4,-1.65),(-3.0,-1.6),(-2.6,-1.43),(-2.2,-1.26),(-1.8,-1.09),(-1.4,-0.82)] { inside.push(line(stage,(x,y),(x,0.3),Palette::faint(),1.2)?); }
  let side=kit.text(stage,"M lies above blue boundary",20.0,Palette::ink(),v(-3.2,-2.5))?;let domain=kit.text(stage,"green chart: box part inside M",21.0,Palette::green(),v(2.7,-2.5))?;
  let int=kit.text(stage,"interior chart: space",24.0,Palette::hero(),v(2.7,1.5))?;let edge=kit.text(stage,"boundary chart: half-space",24.0,Palette::green(),v(2.7,0.65))?; let orient=kit.text(stage,"orientation-preserving coordinates",22.0,Palette::ink(),v(2.7,-0.2))?;
  let mut start=vec![title];start.extend(boundary); fade(stage,&start,true)?;kit.say(stage,"12_1")?;kit.until_sentence(stage,"12_1",2)?;fade(stage,&chart_a,true)?;fade(stage,&[int],true)?;kit.until_sentence(stage,"12_1",3)?;fade(stage,&chart_b,true)?;fade(stage,&inside,true)?;fade(stage,&[edge,orient,side,domain],true)?;
  kit.until_sentence(stage,"12_1",4)?; let weights=kit.text(stage,"smooth weights: supported in charts",22.0,Palette::ink(),v(2.7,-1.0))?;let sum=kit.tex(stage,r"\rho_1+\rho_2+\cdots=1",36.0,Palette::hero(),&[],v(2.7,-1.75))?;fade(stage,&[weights,sum],true)?;kit.hold(stage)?;
  kit.say(stage,"12_2")?;fade(stage,&[int,edge,orient,weights,sum],false)?;let piece=kit.tex(stage,r"\rho_i\,\omega",40.0,Palette::hero(),&[],v(2.8,1.4))?;let inrule=kit.text(stage,"interior: compact support",24.0,Palette::ink(),v(2.8,0.5))?;let brule=kit.text(stage,"boundary: endpoint on half-space",23.0,Palette::green(),v(2.8,-0.4))?;let sign=kit.text(stage,"outward-first orientation",23.0,Palette::ink(),v(2.8,-1.25))?;fade(stage,&[piece],true)?;kit.until_sentence(stage,"12_2",1)?;fade(stage,&[inrule],true)?;kit.until_sentence(stage,"12_2",2)?;fade(stage,&[brule,sign],true)?;kit.finish(stage)
 }
}
