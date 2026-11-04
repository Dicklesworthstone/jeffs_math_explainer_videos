//! Adjacent oriented tiles and signed work. Draft timings require narration review.
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;
use super::opening::{arrow, fade};

pub struct Edges { pub kit: Kit }

impl SceneConstruct for Edges {
    fn name(&self) -> &str { "03_edges" }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let title = kit.text(stage, "Oriented edges add", 34.0, Palette::ink(), v(0.0, 3.0))?;
        let note = kit.text(stage, "Each tile is traversed counterclockwise", 25.0, Palette::dim(), v(0.0, 2.3))?;
        let mut outside = Vec::new();
        for (a,b) in [((-3.0,-0.8),(-0.5,-0.8)),((-0.5,-0.8),(2.0,-0.8)),((2.0,-0.8),(2.0,1.7)),((2.0,1.7),(-0.5,1.7)),((-0.5,1.7),(-3.0,1.7)),((-3.0,1.7),(-3.0,-0.8))] {
            outside.extend(arrow(stage,a,b,Palette::blue())?);
        }
        let mut seam = arrow(stage,(-0.60,-0.8),(-0.60,1.7),Palette::hero())?;
        seam.extend(arrow(stage,(-0.40,1.7),(-0.40,-0.8),Palette::green())?);
        let plus = kit.text(stage,"+ edge",23.0,Palette::hero(),v(-1.25,0.45))?;
        let minus = kit.text(stage,"− edge",23.0,Palette::green(),v(0.25,0.45))?;
        let caption = kit.text(stage,"Shared edge: opposite directions",25.0,Palette::ink(),v(0.0,-1.65))?;
        let mut first = vec![title,note,plus,minus,caption]; first.extend(outside.iter().copied()); first.extend(seam.iter().copied());
        fade(stage,&first,true)?;
        kit.say(stage,"03_1")?; kit.until_sentence(stage,"03_1",3)?;
        let mut disappearing = seam; disappearing.extend([plus,minus,caption]);
        fade(stage,&disappearing,false)?;
        kit.until_sentence(stage,"03_1",4)?;
        let surviving = kit.text(stage,"Only the outside boundary survives",27.0,Palette::green(),v(0.0,-1.65))?;
        fade(stage,&[surviving],true)?; kit.hold(stage)?;
        let mut cleared = vec![note,surviving]; cleared.extend(outside); fade(stage,&cleared,false)?;
        let subtitle = kit.text(stage,"What are we adding along an edge?",27.0,Palette::ink(),v(0.0,2.15))?;
        let field_label = kit.text(stage,"field",24.0,Palette::blue(),v(-3.55,0.7))?;
        let mut field = Vec::new();
        for y in [-0.8_f64,0.0,0.8] { for x in [-3.0_f64,-1.4,0.2,1.8] { field.extend(arrow(stage,(x,y),(x+0.8,y),Palette::blue())?); } }
        let mut forward = arrow(stage,(-1.0,-1.25),(1.4,-1.25),Palette::hero())?;
        let step = kit.text(stage,"step →",26.0,Palette::hero(),v(0.0,-1.8))?;
        let work = kit.text(stage,"Work along step: F · Δr",27.0,Palette::ink(),v(0.0,-2.5))?;
        let positive = kit.text(stage,"positive",26.0,Palette::hero(),v(3.1,-1.25))?;
        let mut second=vec![subtitle,field_label,step,work,positive];second.extend(field.iter().copied());second.extend(forward.iter().copied());fade(stage,&second,true)?;
        kit.say(stage,"03_2")?; kit.until_sentence(stage,"03_2",2)?;
        forward.extend([step,positive]); fade(stage,&forward,false)?;
        let reverse = arrow(stage,(1.4,-1.25),(-1.0,-1.25),Palette::green())?;
        let reverse_label = kit.text(stage,"← reversed step",26.0,Palette::green(),v(0.0,-1.8))?;
        let sign = kit.text(stage,"Reverse the step, reverse its contribution",24.0,Palette::green(),v(0.0,-3.1))?;
        let negative = kit.text(stage,"negative",26.0,Palette::green(),v(3.1,-1.25))?;
        let mut reversal = reverse; reversal.extend([reverse_label,sign,negative]); fade(stage,&reversal,true)?;
        kit.until_sentence(stage,"03_2",3)?; fade(stage,&[sign],false)?;
        let form = kit.text(stage,"One-form: signed contribution to an oriented step",24.0,Palette::ink(),v(0.0,-3.1))?;
        fade(stage,&[form],true)?;
        kit.finish(stage)
    }
}
