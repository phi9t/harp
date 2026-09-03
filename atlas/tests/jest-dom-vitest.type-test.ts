import type { Assertion, AsymmetricMatchersContaining } from "vitest";

type JestDomMatcher = Assertion<HTMLElement>["toBeInTheDocument"];
type AsymmetricMatcher = AsymmetricMatchersContaining["toBeInTheDocument"];

declare const matcher: JestDomMatcher;
declare const asymmetricMatcher: AsymmetricMatcher;

matcher();
asymmetricMatcher();
