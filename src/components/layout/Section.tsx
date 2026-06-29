import type { PropsWithChildren } from "react";
import { Card, CardContent } from "../ui/card";

interface SectionProps extends PropsWithChildren {
  title?: string;
}

export function Section(props: SectionProps) {
  return (
    <Card>
      <CardContent className="space-y-2 p-6">
        {props.title ? (
          <h2 className="mb-2 text-base font-medium">{props.title}</h2>
        ) : null}
        {props.children}
      </CardContent>
    </Card>
  );
}
