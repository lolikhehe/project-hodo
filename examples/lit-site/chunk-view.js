// Static dependency: the TreeWalker part machinery, modelled on Lit's two
// consumers (template part-finder + ChildPart materialisation).
var MARK = '?lit$123$';

// SHOW_ELEMENT (1) | SHOW_COMMENT (128): whitespace text nodes are not walked,
// so marker indices stay stable between the template and the imported clone.
function fillParts(root, values) {
  var walker = document.createTreeWalker(root, 129);
  var positions = [];
  var node;
  var index = 0;
  walker.currentNode = root;
  while ((node = walker.nextNode()) !== null) {
    if (node.nodeType === 8 && node.data === MARK) positions.push(index);
    index++;
  }

  // Re-walk the stamped clone at the very same indices; insert each binding's
  // text after its start marker, exactly like Lit's ChildPart.commitNode.
  walker.currentNode = root;
  node = walker.nextNode();
  index = 0;
  var slots = [];
  for (var p = 0; p < positions.length; p++) {
    while (index < positions[p]) {
      node = walker.nextNode();
      index++;
    }
    var text = document.createTextNode(values[p]);
    node.parentNode.insertBefore(text, node.nextSibling);
    slots.push(text);
  }
  return { positions: positions, slots: slots };
}

export { MARK, fillParts };
