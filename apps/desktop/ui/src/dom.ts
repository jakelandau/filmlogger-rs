// A tiny element-builder helper for the framework-free UI.

type Child = Node | string | number | null | undefined;

export interface Attrs {
  [key: string]: unknown;
}

export function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  attrs?: Attrs,
  ...children: Child[]
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag) as HTMLElementTagNameMap[K];
  if (attrs) {
    for (const [key, value] of Object.entries(attrs)) {
      setAttr(node, key, value);
    }
  }
  append(node, children);
  return node;
}

function setAttr(node: Element, key: string, value: unknown): void {
  if (value == null || value === false) return;
  if (key === "class") node.className = String(value);
  else if (key === "value") (node as HTMLInputElement).value = String(value);
  else if (key === "checked") (node as HTMLInputElement).checked = Boolean(value);
  else if (key === "disabled") (node as HTMLInputElement).disabled = Boolean(value);
  else if (key.startsWith("on") && typeof value === "function") {
    (node as unknown as Record<string, unknown>)[key] = value;
  } else if (value === true) {
    node.setAttribute(key, "");
  } else {
    node.setAttribute(key, String(value));
  }
}

function append(node: ParentNode, children: Child[]): void {
  for (const child of children) {
    if (child == null) continue;
    node.append(child instanceof Node ? child : String(child));
  }
}

export function clear(node: Element): void {
  node.replaceChildren();
}

export function frag(...children: Child[]): DocumentFragment {
  const f = document.createDocumentFragment();
  append(f, children);
  return f;
}
