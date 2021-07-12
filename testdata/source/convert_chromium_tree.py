# Copyright 2021 The AccessKit Authors. All rights reserved.
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# Converts the JSON tree dump produced by a DumpAccessibilityTree test
# in our modified version of Chromium into the AccessKit schema.

import binascii
import json
import re
import sys
import uuid

input_filename = sys.argv[1]
output_filename = sys.argv[2]

# Work around a flaw in the Chromium JSON serializer we're using.
input_encoded = open(input_filename, "rb").read()
def sub_hex(m):
    return binascii.unhexlify(m.group(1))
input_encoded = re.sub(br"%([0-9A-F]{2})", sub_hex, input_encoded)
input_root = json.loads(input_encoded)

nodes = []


def translate_role(input_role):
    renames = {"popUpButton": "popupButton"}
    return renames.get(input_role, input_role)


def add_attr(input_node, node, input_name, *, converter=None):
    renames = {
        "display": "cssDisplay",
        "role": "customRole",
        "text-align": "textAlign",
        "ariaCurrentState": "ariaCurrent",
        "haspopup": "hasPopup",
        "childTreeId": "childTree",
        "tableHeaderId": "tableHeader",
        "tableRowHeaderId": "tableRowHeader",
        "tableColumnHeaderId": "tableColumnHeader",
        "activedescendantId": "activeDescendant",
        "errormessageId": "errorMessage",
        "inPageLinkTargetId": "inPageLinkTarget",
        "memberOfId": "memberOf",
        "nextOnLineId": "nextOnLine",
        "popupForId": "popupFor",
        "previousOnLineId": "previousOnLine",
        "color": "foregroundColor",
        "textOverlineStyle": "overline",
        "textStrikethroughStyle": "strikethrough",
        "textUnderlineStyle": "underline",
        "previousFocusId": "previousFocus",
        "nextFocusId": "nextFocus",
        "indirectChildIds": "indirectChildren",
        "controlsIds": "controls",
        "detailsIds": "details",
        "describedbyIds": "describedBy",
        "flowtoIds": "flowTo",
        "labelledbyIds": "labelledBy",
        "radioGroupIds": "radioGroups",
    }
    name = renames.get(input_name, input_name)
    value = input_node[input_name]
    if converter is not None:
        value = converter(value)
    node[name] = value


def convert_color(rgb_str):
    # TODO: Is this correct?
    return int(rgb_str, 16)


def process_node(input_node):
    node = {"id": input_node["id"], "role": translate_role(input_node["internalRole"])}
    nodes.append(node)

    rect = {
        "left": input_node["boundsX"],
        "top": input_node["boundsY"],
        "width": input_node["boundsWidth"],
        "height": input_node["boundsHeight"],
    }
    bounds = {"rect": rect}
    if (
        input_node.get("boundsOffsetContainerId")
        and input_node["boundsOffsetContainerId"] != nodes[0]["id"]
    ):
        bounds["offsetContainer"] = input_node["boundsOffsetContainerId"]
    node["bounds"] = bounds

    if "actions" in input_node:
        node["actions"] = input_node["actions"].split(",")

    for input_name in (
        "name",
        "nameFrom",
        "description",
        "descriptionFrom",
        "selected",
        "grabbed",
        "characterOffsets",
        "accessKey",
        "autoComplete",
        "checkedState",
        "checkedStateDescription",
        "childTreeId",
        "className",
        "containerLiveRelevant",
        "containerLiveStatus",
        "display",
        "fontFamily",
        "htmlTag",
        "innerHtml",
        "inputType",
        "keyShortcuts",
        "language",
        "liveRelevant",
        "liveStatus",
        "placeholder",
        "role",
        "roleDescription",
        "tooltip",
        "url",
        "defaultActionVerb",
        "sortDirection",
        "ariaCurrentState",
        "haspopup",
        "listStyle",
        "text-align",
        "valueForRange",
        "minValueForRange",
        "maxValueForRange",
        "stepValueForRange",
        "fontSize",
        "fontWeight",
        "textIndent",
        "indirectChildIds",
        "controlsIds",
        "detailsIds",
        "describedbyIds",
        "flowtoIds",
        "labelledbyIds",
        "radioGroupIds",
        "textOverlineStyle",
        "textStrikethroughStyle",
        "textUnderlineStyle",
    ):
        if input_name in input_node:
            add_attr(input_node, node, input_name)

    for input_name in (
        "value",
        "autofillAvailable",
        "default",
        "editable",
        "focusable",
        "hovered",
        "ignored",
        "invisible",
        "linked",
        "multiline",
        "multiselectable",
        "protected",
        "required",
        "visited",
        "busy",
        "nonatomicTextFieldRoot",
        "containerLiveAtomic",
        "containerLiveBusy",
        "liveAtomic",
        "modal",
        "canvasHasFallback",
        "scrollable",
        "clickable",
        "clipsChildren",
        "notUserSelectableStyle",
        "selectedFromFocus",
        "isLineBreakingObject",
        "isPageBreakingObject",
        "hasAriaAttribute",
        "touchPassThrough",
    ):
        if input_node.get(input_name):
            add_attr(input_node, node, input_name)

    for input_name in (
        "scrollX",
        "scrollXMin",
        "scrollXMax",
        "scrollY",
        "scrollYMin",
        "scrollYMax",
        "ariaColumnCount",
        "ariaCellColumnIndex",
        "ariaCellColumnSpan",
        "ariaRowCount",
        "ariaCellRowIndex",
        "ariaCellRowSpan",
        "tableRowCount",
        "tableColumnCount",
        "tableHeaderId",
        "tableRowIndex",
        "tableRowHeaderId",
        "tableColumnIndex",
        "tableColumnHeaderId",
        "tableCellColumnIndex",
        "tableCellColumnSpan",
        "tableCellRowIndex",
        "tableCellRowSpan",
        "hierarchicalLevel",
        "activedescendantId",
        "errormessageId",
        "inPageLinkTargetId",
        "memberOfId",
        "nextOnLineId",
        "popupForId",
        "previousOnLineId",
        "setSize",
        "posInSet",
        "previousFocusId",
        "nextFocusId",
    ):
        if input_name in input_node:
            add_attr(input_node, node, input_name, converter=int)

    for input_name in ("colorValue", "backgroundColor", "color"):
        if input_name in input_node:
            add_attr(input_node, node, input_name, converter=convert_color)

    if input_node.get("expanded"):
        node["expanded"] = True
    elif input_node.get("collapsed"):
        node["expanded"] = False

    for input_name in ("horizontal", "vertical"):
        if input_node.get(input_name):
            node["orientation"] = input_name
            break

    if "invalidState" in input_node:
        if input_node["invalidState"] == "other":
            node["invalidState"] = {"other": input_node["ariaInvalidValue"]}
        else:
            add_attr(input_node, node, "invalidState")

    if "restriction" in input_node:
        node[input_node["restriction"]] = True

    if "textStyle" in input_node:
        text_style = int(input_node["textStyle"])
        if text_style & 2:
            node["bold"] = True
        if text_style & 4:
            node["italic"] = True

    if "wordStarts" in input_node:
        node["words"] = []
        for i, start in enumerate(input_node["wordStarts"]):
            end = input_node["wordEnds"][i]
            node["words"].append({"start": start, "end": end})

    if input_node.get("children"):
        node["children"] = []
        for input_child in input_node["children"]:
            child = process_node(input_child)
            node["children"].append(child["id"])

    return node


root = process_node(input_root)
namespace_uuid = uuid.UUID("6a529f27-3bc6-4609-80a6-370f5fd07030")
tree = {
    "id": str(uuid.uuid5(namespace_uuid, output_filename)),
    "sourceStringEncoding": "utf16",
}
# TODO: Other tree attributes?
tree_update = {"nodes": nodes, "tree": tree, "root": root["id"]}
with open(output_filename, "w") as f:
    json.dump(tree_update, f, indent="  ")
