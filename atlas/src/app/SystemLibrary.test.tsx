import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";

import { AtlasApp } from "./AtlasApp";

describe("system reading library", () => {
  beforeEach(() => {
    window.location.hash = "#systems";
  });

  it("lists all sixteen published system readings", () => {
    render(<AtlasApp />);

    const table = screen.getByRole("table", { name: "Technical system readings" });
    expect(table).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Open AFlow system reading" }),
    ).toBeInTheDocument();
    expect(screen.getAllByText("published")).toHaveLength(16);
    expect(screen.getByText("Automated Design of Agentic Systems")).toBeInTheDocument();
    expect(screen.queryByText("planned")).not.toBeInTheDocument();
  });
});
