//! Disk and cap with common boundary. Draft projection, timing and review.
use explainer_kit::{Timed,Kit, Palette, ink_style, v};
use fmn::prelude::*;
use super::opening::{arrow, fade, line};

pub struct Cap { pub kit: Kit }
fn projected(r:f64,t:f64,raised:bool)->(f64,f64) {
    let x=r*t.cos();let y=r*t.sin();let z=if raised {1.0-r*r} else {0.0};
    (-2.0+1.7*x-y,0.5*x+0.6*y+1.5*z-0.35)
}
fn mesh(stage:&mut Stage<'_>,raised:bool,color:Srgb)->fmn::Result<Vec<Mob>> {
    let mut mobs=Vec::new();let tau=std::f64::consts::TAU;
    for radius in [0.25,0.5,0.75] {for i in 0..24 {let a=tau*(i as f64)/24.0;let b=tau*((i+1) as f64)/24.0;mobs.push(line(stage,projected(radius,a,raised),projected(radius,b,raised),color,1.5)?);}}
    for i in 0..8 {let t=tau*(i as f64)/8.0;for j in 0..8 {mobs.push(line(stage,projected((j as f64)/8.0,t,raised),projected(((j+1) as f64)/8.0,t,raised),color,1.5)?);}}
    Ok(mobs)
}
impl SceneConstruct for Cap {
 fn name(&self)->&str {"05_curved_surface"}
 fn construct(&mut self,stage:&mut Stage<'_>)->fmn::Result<()> {
  let kit=self.kit.clone();let tau=std::f64::consts::TAU;
  let title=kit.text(stage,"One rim, two surfaces",34.0,Palette::ink(),v(0.0,3.1))?;
  let mut rim=Vec::new();for i in 0..32 {let a=tau*(i as f64)/32.0;let b=tau*((i+1) as f64)/32.0;rim.push(line(stage,projected(1.0,a,false),projected(1.0,b,false),Palette::hero(),3.0)?);}
  let disk=mesh(stage,false,Palette::blue())?;let cap=mesh(stage,true,Palette::green())?;
  let disk_label=kit.text(stage,"flat disk",25.0,Palette::blue(),v(-2.0,-2.4))?;
  let cap_label=kit.text(stage,"cap: z = 1 − x² − y²",25.0,Palette::green(),v(-2.0,-2.4))?;
  let mut first=vec![title,disk_label];first.extend(rim.iter().copied());first.extend(disk.iter().copied());fade(stage,&first,true)?;
  kit.say(stage,"05_1")?;
  // Transform corresponding mesh segments together; the shared rim is separate and fixed.
  assert_eq!(disk.len(),cap.len());
  let mut lift:Vec<Box<dyn Animation>>=Vec::new();
  for (&from,&to) in disk.iter().zip(&cap) {lift.push(stage.prepare(Transform::new(from,to).rt(2.0))?);}
  stage.play_prepared(lift)?;
  fade(stage,&[disk_label],false)?;fade(stage,&[cap_label],true)?;
  kit.until_sentence(stage,"05_1",2)?;
  let mut normal=arrow(stage,(-2.0,1.15),(-2.0,2.2),Palette::green())?;
  let normal_label=kit.text(stage,"upward normal",22.0,Palette::green(),v(-0.15,2.25))?;normal.push(normal_label);fade(stage,&normal,true)?;
  let mut directed=Vec::new();for t in [0.3,2.4,4.5] {directed.extend(arrow(stage,projected(1.0,t,false),projected(1.0,t+0.35,false),Palette::hero())?);}fade(stage,&directed,true)?;kit.hold(stage)?;
  kit.say(stage,"05_2")?;
  let field=kit.text(stage,"F = (−y/2, x/2, 0)",27.0,Palette::ink(),v(3.0,1.8))?;fade(stage,&[field],true)?;
  kit.until_sentence(stage,"05_2",1)?;let curl=kit.text(stage,"curl F = (0,0,1)",26.0,Palette::blue(),v(3.0,1.0))?;fade(stage,&[curl],true)?;
  kit.until_sentence(stage,"05_2",2)?;
  let start=projected(1.0,0.0,false);
  let point=stage.arena_mut().add(Circle::new().radius(0.09).style(ink_style(Palette::hero())).build().moved_to(v(start.0,start.1)));
  fade(stage,&[point],true)?;
  for i in 1..=32 {let p=projected(1.0,tau*(i as f64)/32.0,false);let animation=point.animate().set_anim_args(AnimateArgs {run_time:Some(0.08),..AnimateArgs::default()})?.move_to(v(p.0,p.1),ORIGIN)?;let animation=stage.prepare(animation)?;stage.play_prepared(vec![animation])?;}
  let circulation=kit.text(stage,"rim circulation = π",26.0,Palette::hero(),v(3.0,0.15))?;fade(stage,&[circulation],true)?;
  kit.until_sentence(stage,"05_2",3)?;let flux=kit.text(stage,"curl flux = π",26.0,Palette::green(),v(3.0,-0.7))?;let note=kit.text(stage,"same oriented boundary",22.0,Palette::ink(),v(3.0,-1.55))?;fade(stage,&[flux,note],true)?;
  kit.finish(stage)
 }
}
#[cfg(test)]
mod tests {
 use super::projected;
 #[test] fn rim_is_fixed_during_lift(){for i in 0..32 {let t=std::f64::consts::TAU*(i as f64)/32.0;let a=projected(1.0,t,false);let b=projected(1.0,t,true);assert!((a.0-b.0).abs()<1e-12&&(a.1-b.1).abs()<1e-12);}}
}
