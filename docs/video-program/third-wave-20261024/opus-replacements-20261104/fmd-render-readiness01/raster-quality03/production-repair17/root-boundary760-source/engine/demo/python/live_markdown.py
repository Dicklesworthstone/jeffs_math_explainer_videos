"""Native source-aware document edits, code and tables; no external assets."""
from manimlib import *
from fmn_python.markdown import MarkdownMobject


class LiveMarkdown(Scene):
    def construct(self):
        initial = '# Native documents\n\n**Edit** the source, animate the result.\n\n```rust\nlet value = 1;\n```\n'
        updated = '# Native documents\n\n**Keep** the scene, change the content.\n\n```rust\nlet value = 8;\n```\n'
        document = MarkdownMobject(initial, font_size=28).to_edge(UP)
        self.add(document)
        self.play(document.animate_source(updated), run_time=1.5)
        self.play(document.select_text('let value')[0].animate.shift(RIGHT * .3), run_time=.5)
        document.set_source(updated + '\n| State | Value |\n| --- | --- |\n| ready | 8 |\n')
        self.wait(.5)
