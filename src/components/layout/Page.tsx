import type { PropsWithChildren } from "react";

interface PageProps extends PropsWithChildren {
  title: string;
  description?: string;
}

export function Page(props: PageProps) {
  return (
    <div className="mx-auto flex max-w-3xl flex-col gap-6">
      <section>
        <h1 className="text-2xl font-semibold">{props.title}</h1>
        {props.description ? (
          <p className="mt-2 text-sm text-muted-foreground">
            {props.description}
          </p>
        ) : null}
      </section>
      {props.children}
    </div>
  );
}
