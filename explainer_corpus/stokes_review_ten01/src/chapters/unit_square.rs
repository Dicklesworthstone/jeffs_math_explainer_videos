//! A unit-square one-form example; unbuilt draft, all pacing provisional.
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;
use super::opening::{arrow, fade};

pub struct UnitSquare { pub kit: Kit }

impl SceneConstruct for UnitSquare {
    fn name(&self) -> &str { "04_unit_square" }
    fn construct(&mut self,stage:&mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let title=kit.text(stage,"A one-form on the unit square",34.0,Palette::ink(),v(0.0,3.1))?;
        let form=kit.text(stage,"ω = x dy",32.0,Palette::hero(),v(2.5,2.2))?;
        let question=kit.text(stage,"Which edge contributes?",27.0,Palette::ink(),v(2.5,1.35))?;
        let mut ccw=Vec::new();
        for (a,b) in [((-3.5,-1.0),(-0.5,-1.0)),((-0.5,-1.0),(-0.5,2.0)),((-0.5,2.0),(-3.5,2.0)),((-3.5,2.0),(-3.5,-1.0))] {ccw.extend(arrow(stage,a,b,Palette::blue())?);}
        let labels=[kit.text(stage,"(0,0)",23.0,Palette::dim(),v(-3.8,-1.4))?,kit.text(stage,"(1,0)",23.0,Palette::dim(),v(-0.2,-1.4))?,kit.text(stage,"(1,1)",23.0,Palette::dim(),v(-0.2,2.4))?,kit.text(stage,"(0,1)",23.0,Palette::dim(),v(-3.8,2.4))?];
        let mut first=vec![title,form,question];first.extend(ccw.iter().copied());first.extend(labels);fade(stage,&first,true)?;
        kit.say(stage,"04_1")?;kit.hold(stage)?;kit.mark(stage,"predict");stage.wait(3.0)?;
        kit.say(stage,"04_2")?;fade(stage,&[question],false)?;
        let horizontal=kit.text(stage,"horizontal: dy = 0",24.0,Palette::dim(),v(2.5,1.25))?;
        let left=kit.text(stage,"left: x = 0",24.0,Palette::dim(),v(2.5,0.6))?;
        let right=kit.text(stage,"right: x = 1, Δy = 1",24.0,Palette::hero(),v(2.5,-0.05))?;
        let result=kit.text(stage,"∮ ω = 1",30.0,Palette::green(),v(2.5,-0.9))?;
        let area=kit.text(stage,"dω = dx ∧ dy: unit area = 1",25.0,Palette::ink(),v(0.0,-2.35))?;
        // Sentence boundaries are provisional until recorded speech has been reviewed.
        fade(stage,&[horizontal],true)?;
        kit.until_sentence(stage,"04_2",1)?; fade(stage,&[left],true)?;
        kit.until_sentence(stage,"04_2",2)?;
        let highlight=arrow(stage,(-0.5,-1.0),(-0.5,2.0),Palette::hero())?;
        let mut right_reveal=highlight.clone(); right_reveal.extend([right,result]); fade(stage,&right_reveal,true)?;
        kit.until_sentence(stage,"04_2",3)?; fade(stage,&[area],true)?;kit.hold(stage)?;
        kit.say(stage,"04_3")?;fade(stage,&highlight,false)?;fade(stage,&ccw,false)?;
        let mut cw=Vec::new();
        for (a,b) in [((-0.5,-1.0),(-3.5,-1.0)),((-0.5,2.0),(-0.5,-1.0)),((-3.5,2.0),(-0.5,2.0)),((-3.5,-1.0),(-3.5,2.0))] {cw.extend(arrow(stage,a,b,Palette::green())?);}
        fade(stage,&[horizontal,left,right,result,area],false)?;
        let reverse=kit.text(stage,"Reverse orientation",27.0,Palette::green(),v(2.5,1.25))?;
        let boundary=kit.text(stage,"boundary integral = −1",25.0,Palette::ink(),v(2.5,0.35))?;
        let interior=kit.text(stage,"oriented area integral = −1",25.0,Palette::ink(),v(2.5,-0.4))?;
        let mut final_view=cw;final_view.extend([reverse,boundary,interior]);fade(stage,&final_view,true)?;
        kit.finish(stage)
    }
}
