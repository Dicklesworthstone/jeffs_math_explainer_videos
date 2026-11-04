//! Three-dimensional boundary cancellation and unit-ball divergence example.
//! Draft projection and timing; not yet built or reviewed.
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;
use super::opening::{arrow,fade,line};

pub struct Volume { pub kit:Kit }
fn cube_point(x:f64,y:f64,z:f64)->(f64,f64){(-3.5+1.6*x-0.8*y,-0.9+0.30*x+0.45*y+1.6*z)}
fn sphere_point(x:f64,y:f64,z:f64)->(f64,f64){(-2.2+1.65*x+0.4*y,0.25+0.35*y+1.65*z)}
fn segment(stage:&mut Stage<'_>,a:(f64,f64,f64),b:(f64,f64,f64),color:Srgb)->fmn::Result<Mob>{line(stage,cube_point(a.0,a.1,a.2),cube_point(b.0,b.1,b.2),color,2.2)}
fn sphere_mesh(stage:&mut Stage<'_>)->fmn::Result<Vec<Mob>> {
 let mut m=Vec::new();let tau=std::f64::consts::TAU;
 for lat in [-0.65_f64,0.0,0.65] {let z=lat.sin();let r=lat.cos();for i in 0..24 {let a=tau*(i as f64)/24.0;let b=tau*((i+1) as f64)/24.0;m.push(line(stage,sphere_point(r*a.cos(),r*a.sin(),z),sphere_point(r*b.cos(),r*b.sin(),z),Palette::blue(),1.5)?);}}
 for lon in [0.0_f64,std::f64::consts::FRAC_PI_4,std::f64::consts::FRAC_PI_2,3.0*std::f64::consts::FRAC_PI_4] {for i in 0..24 {let a=tau*(i as f64)/24.0;let b=tau*((i+1) as f64)/24.0;m.push(line(stage,sphere_point(a.cos()*lon.cos(),a.cos()*lon.sin(),a.sin()),sphere_point(b.cos()*lon.cos(),b.cos()*lon.sin(),b.sin()),Palette::blue(),1.5)?);}}
 Ok(m)
}
impl SceneConstruct for Volume {
 fn name(&self)->&str {"06_volume"}
 fn construct(&mut self,stage:&mut Stage<'_>)->fmn::Result<()> {
  let kit=self.kit.clone();let title=kit.text(stage,"From volume to surface",34.0,Palette::ink(),v(0.0,3.0))?;
  let mut shell=Vec::new();
  for x in [0.0_f64,2.0] {for (a,b) in [((x,0.,0.),(x,1.,0.)),((x,1.,0.),(x,1.,1.)),((x,1.,1.),(x,0.,1.)),((x,0.,1.),(x,0.,0.))] {shell.push(segment(stage,a,b,Palette::blue())?);}}
  for y in [0.0_f64,1.0] {for z in [0.0_f64,1.0] {shell.push(segment(stage,(0.,y,z),(2.,y,z),Palette::blue())?);}}
  // Representative outward arrows on the exterior faces.
  for (a,b) in [((0.0,0.5,0.5),(-0.35,0.5,0.5)),((2.0,0.5,0.5),(2.35,0.5,0.5)),((0.5,0.5,1.0),(0.5,0.5,1.3)),((1.5,0.5,1.0),(1.5,0.5,1.3))] {shell.extend(arrow(stage,cube_point(a.0,a.1,a.2),cube_point(b.0,b.1,b.2),Palette::blue())?);}
  let mut shared=Vec::new();let x=1.0_f64;
  for (a,b) in [((x,0.,0.),(x,1.,0.)),((x,1.,0.),(x,1.,1.)),((x,1.,1.),(x,0.,1.)),((x,0.,1.),(x,0.,0.))] {shared.push(segment(stage,a,b,Palette::hero())?);}
  shared.extend(arrow(stage,cube_point(1.0,0.45,0.5),cube_point(0.65,0.45,0.5),Palette::green())?);shared.extend(arrow(stage,cube_point(1.0,0.55,0.5),cube_point(1.35,0.55,0.5),Palette::hero())?);
  let seam=kit.text(stage,"Shared face: opposite outward directions",24.0,Palette::ink(),v(0.0,-2.15))?;
  let mut first=vec![title,seam];first.extend(shell.iter().copied());first.extend(shared.iter().copied());fade(stage,&first,true)?;
  kit.say(stage,"06_1")?;kit.until_sentence(stage,"06_1",4)?;fade(stage,&shared,false)?;fade(stage,&[seam],false)?;
  let outer=kit.text(stage,"Internal flux cancels; outer surface remains",24.0,Palette::green(),v(0.0,-2.15))?;fade(stage,&[outer],true)?;kit.hold(stage)?;
  let mut remove=shell;remove.push(outer);fade(stage,&remove,false)?;
  let sphere=sphere_mesh(stage)?;let field=kit.text(stage,"F = (x,y,z)/3",28.0,Palette::ink(),v(2.8,1.9))?;let mut second=sphere;second.push(field);
  for (x,z) in [(1.0_f64,0.0_f64),(-1.0,0.0),(0.0,1.0),(0.0,-1.0)] {second.extend(arrow(stage,sphere_point(x,0.0,z),sphere_point(1.25*x,0.0,1.25*z),Palette::hero())?);}
  fade(stage,&second,true)?;
  kit.say(stage,"06_2")?;kit.until_sentence(stage,"06_2",1)?;
  let div=kit.text(stage,"div F = 1",26.0,Palette::blue(),v(2.8,1.15))?;fade(stage,&[div],true)?;
  kit.until_sentence(stage,"06_2",2)?;let volume=kit.text(stage,"volume integral = 4π/3",25.0,Palette::hero(),v(2.8,0.35))?;fade(stage,&[volume],true)?;
  kit.until_sentence(stage,"06_2",3)?;let normal=kit.text(stage,"on unit sphere: F · n = 1/3",24.0,Palette::ink(),v(2.8,-0.5))?;let flux=kit.text(stage,"surface flux = (1/3)(4π) = 4π/3",24.0,Palette::green(),v(2.8,-1.25))?;fade(stage,&[normal,flux],true)?;
  kit.finish(stage)
 }
}
