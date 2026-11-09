from pathlib import Path
import json,subprocess,sys
R=Path.cwd();B=R/'narration_lab/third-wave-20261024/root/five-film-release-20261027'
c=sys.argv[1];J=B/('algebra-final10-'+c);T=B/'algebra-fixes-timeline04/timeline';S=B/'algebra-fixes-score07/score/stems';N=B/'algebra-fixes-assemble03/narration';I=B/'algebra-fixes-inputs02'
a=json.loads((J/'resource-receipt.json').read_text());assert a['admitted'] and 'exit' not in a
assert json.loads((S/'qa'/(c+'.json')).read_text())['passed'] is True
out=J/'render';assert not out.exists()
subprocess.run([str(I/'abstract_algebra_expanded'),'render',c,'--res','3840x2160','--fps','60','--threads','4','--narration',str(N),'--score',str(S),'--out',str(out),'--lint','--crf','16','--preset','veryfast'],check=True,timeout=14400)
for suffix in ['.timeline.tsv','.cues.tsv']:assert (out/(c+suffix)).read_bytes()==(T/(c+suffix)).read_bytes()
v=next(x for x in json.loads(subprocess.check_output(['ffprobe','-v','error','-show_streams','-of','json',str(out/(c+'.mp4'))]))['streams'] if x['codec_type']=='video')
frames=next(int(x.split('\t')[1]) for x in (T/(c+'.timeline.tsv')).read_text().splitlines() if x.startswith('frames\t'))
assert (v['width'],v['height'],v['avg_frame_rate'],int(v['nb_frames']))==(3840,2160,'60/1',frames)
(J/'technical-check.json').write_text(json.dumps(dict(chapter=c,frames=frames,resolution='3840x2160',fps=60,timeline_and_cues_equal=True,final_picture_review=False,motion_review=False,listening_accepted=False,publication_accepted=False),indent=2))
