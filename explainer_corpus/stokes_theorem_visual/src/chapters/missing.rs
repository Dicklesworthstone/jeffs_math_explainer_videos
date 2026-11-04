//! Omitted-coordinate notation candidate; unreviewed and not production.
use explainer_kit::{Kit, Palette, v};
use fmn::prelude::*;
use super::opening::{arrow, fade};
pub struct Missing { pub kit: Kit }
impl SceneConstruct for Missing {
    fn name(&self) -> &str { "09_missing_coordinate" }
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        let title = kit.text(stage, "One term for each missing direction", 34.0, Palette::ink(), v(0.0,3.0))?;
        let heading = kit.text(stage, "Three dimensions: choose an omitted coordinate", 27.0, Palette::ink(), v(0.0,2.1))?;
        let row1 = kit.text(stage, "omit x:   f₁ dy ∧ dz", 28.0, Palette::blue(), v(0.0,1.1))?;
        let row2 = kit.text(stage, "omit y:  −f₂ dx ∧ dz", 28.0, Palette::hero(), v(0.0,0.3))?;
        let row3 = kit.text(stage, "omit z:   f₃ dx ∧ dy", 28.0, Palette::green(), v(0.0,-0.5))?;
        fade(stage, &[title,heading], true)?;
        kit.say(stage, "09_1")?;
        kit.until_sentence(stage, "09_1", 1)?; fade(stage, &[row1], true)?;
        kit.until_sentence(stage, "09_1", 2)?; fade(stage, &[row2,row3], true)?;
        let note = kit.text(stage, "Signs encode the face orientations", 24.0, Palette::dim(), v(0.0,-1.5))?;
        kit.until_sentence(stage, "09_1", 3)?; fade(stage, &[note], true)?;
        kit.hold(stage)?;
        kit.say(stage, "09_2")?; fade(stage, &[row1,row3,heading,note], false)?;
        let equal = kit.text(stage, "−f₂ dx ∧ dz   =   +f₂ dz ∧ dx", 30.0, Palette::hero(), v(0.0,1.7))?;
        let swap = kit.text(stage, "Swap directions → reverse the sign", 26.0, Palette::ink(), v(0.0,0.7))?;
        let mut u = arrow(stage,(-1.5,-0.8),(0.0,-0.8),Palette::blue())?;
        u.extend(arrow(stage,(-1.5,-0.8),(-1.5,0.2),Palette::green())?);
        let dx_first = kit.text(stage,"1st: dx",21.0,Palette::blue(),v(-0.6,-1.3))?;
        let dz_second = kit.text(stage,"2nd: dz",21.0,Palette::green(),v(-2.7,-0.3))?;
        let dx_second = kit.text(stage,"2nd: dx",21.0,Palette::blue(),v(-0.6,-1.3))?;
        let dz_first = kit.text(stage,"1st: dz",21.0,Palette::green(),v(-2.7,-0.3))?;
        let mut show=vec![equal,swap,dx_first,dz_second]; show.extend(u.iter().copied()); fade(stage,&show,true)?;
        kit.until_sentence(stage,"09_2",1)?; fade(stage,&[dx_first,dz_second],false)?; fade(stage,&[dx_second,dz_first],true)?;
        kit.hold(stage)?;
        kit.say(stage,"09_3")?; let mut clear=vec![row2,equal,swap,dx_second,dz_first]; clear.extend(u); fade(stage,&clear,false)?;
        let general = kit.tex(stage, r"\omega=\sum_i(-1)^{i-1}f_i", 42.0, Palette::ink(), &[], v(0.0,1.7))?;
        let wedge = kit.tex(stage, r"dx^1\wedge\cdots\wedge\widehat{dx^i}\wedge\cdots\wedge dx^n", 42.0, Palette::ink(), &[], v(0.0,0.85))?;
        let legend = kit.text(stage, "Hat means: omit this factor", 23.0, Palette::dim(), v(0.0,0.2))?;
        let df = kit.tex(stage, r"df_i=\sum_j(\partial_j f_i)\,dx^j", 42.0, Palette::blue(), &[], v(0.0,-0.6))?;
        let question = kit.text(stage,"Which terms survive the wedge?",27.0,Palette::hero(),v(0.0,-1.6))?;
        fade(stage,&[general,wedge,legend,df],true)?;
        kit.until_sentence(stage,"09_3",2)?; fade(stage,&[question],true)?;
        kit.hold(stage)?;
        kit.mark(stage,"predict"); stage.wait(3.0)?; kit.finish(stage)
    }
}
