import type { PropsWithChildren } from "react";

export function AppContent(props: PropsWithChildren) {
  return (
    <section className="min-h-0 flex-1 overflow-auto p-6">
      {props.children}
    </section>
  );
}
