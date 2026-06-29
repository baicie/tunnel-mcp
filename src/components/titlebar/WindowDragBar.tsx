import {
  getDragRegionAttrs,
  getDragRegionStyle,
  type WindowChromePolicy,
} from "../../lib/platform/windowChrome";

interface WindowDragBarProps {
  chrome: WindowChromePolicy;
}

export function WindowDragBar({ chrome }: WindowDragBarProps) {
  if (chrome.dragBarHeight <= 0) {
    return null;
  }

  return (
    <div
      className="fixed left-0 right-0 top-0 z-[70]"
      {...getDragRegionAttrs(chrome.enableDragRegion)}
      style={{
        height: chrome.dragBarHeight,
        ...getDragRegionStyle(chrome.enableDragRegion),
      }}
    />
  );
}
