"""Live numeric spans over Scribe's byte map and Marionette's native families.

Only source/path bookkeeping lives here. DecimalNumber owns native number
layout, and the existing mobject operations own placement, style, adoption,
copying and animation. No expression is re-typeset or re-rendered to find it.
"""
from __future__ import annotations

from bisect import bisect_right
import operator
from typing import Any

_TOKEN = r"\decimalmob"


def _rewrite(source, edits):
    chunks, cursor = [], 0
    for start, end in edits:
        if start < cursor or end <= start:
            raise ValueError("numeric TeX selections overlap or are empty")
        chunks.extend((source[cursor:start], _TOKEN))
        cursor = end
    chunks.append(source[cursor:])
    return "".join(chunks)


def _rewrite_parts(tex, source, edits, rewritten):
    parts = list(tex.tex_strings)
    separator = getattr(tex, "_tex_arg_separator", " ")
    if separator.join(parts) != source:
        raise ValueError("Tex argument metadata disagrees with its source")
    result, offset = [], 0
    for part in parts:
        end = offset + len(part)
        chunks, cursor = [], offset
        for start, stop in edits:
            if stop <= offset or start >= end:
                continue
            left, right = max(offset, start), min(end, stop)
            chunks.append(source[cursor:left])
            if offset <= start < end:
                chunks.append(_TOKEN)
            cursor = right
        chunks.append(source[cursor:end])
        result.append("".join(chunks))
        offset = end + len(separator)
    if separator.join(result) != rewritten:
        raise ValueError("numeric replacement cannot remove a TeX argument separator")
    return result


def _source_map(source, edits):
    boundaries, count = [0], 0
    for char in source:
        count += len(char.encode("utf-8"))
        boundaries.append(count)
    rows, shift = [], 0
    for start, end in edits:
        low, high = boundaries[start], boundaries[end]
        new_low = low + shift
        shift += len(_TOKEN) - (high - low)
        rows.append((low, high, new_low, high + shift, shift))
    starts = [row[0] for row in rows]

    def boundary(value, *, right=False):
        index = bisect_right(starts, value) - 1
        if index < 0:
            return value
        low, high, new_low, new_high, shift = rows[index]
        if value == low:
            return new_low
        if value < high:
            return new_high if right else new_low
        return value + shift

    return set(boundaries), rows, boundary


def _resolve(tex, paths):
    """Freeze native parents before edits; keep unmapped decorations intact."""
    if len(set(paths)) != len(paths):
        raise ValueError("Tex span map contains duplicate family paths")
    ordered = sorted(paths)
    for left, right in zip(ordered, ordered[1:]):
        if right[:len(left)] == left:
            raise ValueError("Tex span map contains overlapping family paths")
    nodes, children = {(): tex}, {}
    for path in paths:
        prefix = ()
        for index in path:
            if isinstance(index, bool) or not isinstance(index, int) or index < 0:
                raise ValueError("Tex span map contains an invalid family index")
            if prefix not in children:
                children[prefix] = tuple(nodes[prefix].submobjects)
            if index >= len(children[prefix]):
                raise ValueError("Tex span map no longer matches its native family")
            following = (*prefix, index)
            nodes[following] = children[prefix][index]
            prefix = following
    return nodes, children


def _note(error, cleanup):
    try:
        error.add_note("live TeX rollback also failed: " + type(cleanup).__name__)
    except BaseException:
        pass


def install_live_tex(native: Any) -> None:
    """Bind the existing Tex class, preserving all qualified class identities."""
    g = vars(native)
    if g.get("_FMN_LIVE_TEX_INSTALLED", False):
        return
    Tex, Decimal, VGroup, VMobject = (g[name] for name in ("Tex", "DecimalNumber", "VGroup", "VMobject"))

    def make_number_changeable(self, value, index=0, replace_all=False, **config):
        substring = str(value)
        index = operator.index(index)
        occurrences = [
            (span, tuple(ordinals))
            for span, ordinals in zip(self.find_spans_by_selector(substring),
                                      self._selected_string_ordinals(substring))
            if ordinals
        ]
        if not occurrences or index >= len(occurrences):
            return VMobject()
        selected = occurrences if replace_all else [occurrences[index]]
        selected = sorted(selected, key=lambda entry: entry[0])
        source = self.string
        spans = tuple(tuple(span) for span in self._string_sub_spans)
        paths = tuple(tuple(path) for path in self._string_sub_paths)
        if len(spans) != len(paths):
            raise ValueError("Tex source spans and family paths have different lengths")
        nodes, children = _resolve(self, paths)
        edits = [span for span, _ in selected]
        rewritten = _rewrite(source, edits)
        parts = _rewrite_parts(self, source, edits, rewritten)
        boundaries, mapped, boundary = _source_map(source, edits)
        if any(len(span) != 2 or span[0] not in boundaries or span[1] not in boundaries
               or span[0] > span[1] for span in spans):
            raise ValueError("Tex source spans must be valid UTF-8 byte intervals")
        for _, ordinals in selected:
            selected_spans = [spans[ordinal] for ordinal in ordinals]
            if len(set(selected_spans)) != len(selected_spans):
                # A macro may materialize one argument in several positions.
                # One DecimalNumber cannot replace those independent copies:
                # merging their bounding boxes silently deletes mathematics.
                raise ValueError(
                    "numeric TeX source expands into multiple native occurrences; "
                    "use separately authored numeric spans for independent live readouts"
                )
        claimed = [ordinal for _, ordinals in selected for ordinal in ordinals]
        if len(set(claimed)) != len(claimed):
            raise ValueError("numeric TeX selections share a native source primitive")
        if paths == ((),) and edits != [(0, len(source))]:
            raise ValueError("a whole-entry Tex span requires a whole-source numeric replacement")

        if "num_decimal_places" not in config:
            config["num_decimal_places"] = len(substring.split(".", 1)[1]) if "." in substring else 0
        # Build every replacement and its detached placement/style before
        # changing a live parent. A later formatting error cannot splice half
        # of a replace_all request into the scene.
        replacements, by_first, removed, actions = [], {}, set(), {}
        for row, (_, ordinals) in zip(mapped, selected):
            first = ordinals[0]
            decimal = Decimal(float(value), **config)
            part = VGroup(*(nodes[paths[ordinal]].copy() for ordinal in ordinals))
            decimal.replace(part)
            # A temporary VGroup's default color is not the selected glyph's
            # color. Read style from the actual first source primitive.
            decimal.match_style(nodes[paths[first]])
            replacements.append(decimal)
            by_first[first] = (decimal, (row[2], row[3]))
            removed.update(ordinals[1:])
            for ordinal in ordinals:
                actions[paths[ordinal]] = decimal if ordinal == first else None

        root_replacement = () in actions
        plans, remaps = {}, {}
        if root_replacement:
            plans[()] = (self, tuple(self.submobjects), (replacements[0],))
        else:
            for path in actions:
                parent = path[:-1]
                if parent in plans:
                    continue
                old, new, indices = children[parent], [], {}
                for old_index, child in enumerate(old):
                    child_path = (*parent, old_index)
                    replacement = actions.get(child_path, child)
                    if replacement is not None:
                        indices[old_index] = len(new)
                        new.append(replacement)
                plans[parent] = (nodes[parent], old, tuple(new))
                remaps[parent] = indices
            # One shared parent cannot accept two different splices through
            # separate aliases. Refuse before changing either path.
            edited_ids = {id(parent) for parent, _, _ in plans.values()}
            seen_ids = set()
            for node in nodes.values():
                marker = id(node)
                if marker in edited_ids and marker in seen_ids:
                    raise ValueError("numeric TeX edits alias a shared parent through multiple paths")
                seen_ids.add(marker)

        new_spans, new_paths = [], []
        for ordinal, (span, path) in enumerate(zip(spans, paths)):
            if ordinal in removed:
                continue
            replacement = by_first.get(ordinal)
            new_spans.append(replacement[1] if replacement else
                             (boundary(span[0]), boundary(span[1], right=True)))
            if root_replacement:
                new_paths.append([0])
            else:
                new_paths.append([remaps.get(path[:i], {}).get(index, index)
                                  for i, index in enumerate(path)])
        result = VGroup(*replacements) if replace_all else replacements[0]
        old_metadata = {name: self.__dict__[name] for name in (
            "string", "tex_string", "tex_strings", "_string_sub_spans", "_string_sub_paths",
        )}
        root_points = self.get_points().copy() if root_replacement else None
        attempted = []
        try:
            if root_replacement:
                self.set_points(root_points[:0])
            for path in sorted(plans, key=lambda item: (-len(item), item)):
                parent, old, new = plans[path]
                attempted.append((parent, old))
                parent.set_submobjects(list(new))
            self.__dict__.update(string=rewritten, tex_string=rewritten, tex_strings=parts,
                                 _string_sub_spans=new_spans, _string_sub_paths=new_paths)
        except BaseException as error:
            # Include the failing parent: its authored override may raise
            # after completing the native splice. Never hide the first error.
            for parent, old in reversed(attempted):
                try:
                    parent.set_submobjects(list(old))
                except BaseException as cleanup:
                    _note(error, cleanup)
            if root_points is not None:
                try:
                    self.set_points(root_points)
                except BaseException as cleanup:
                    _note(error, cleanup)
            self.__dict__.update(old_metadata)
            raise
        return result

    make_number_changeable.__name__ = "make_number_changeable"
    make_number_changeable.__qualname__ = Tex.__qualname__ + ".make_number_changeable"
    make_number_changeable.__module__ = Tex.__module__
    Tex.make_number_changeable = make_number_changeable
    g["_FMN_LIVE_TEX_INSTALLED"] = True
