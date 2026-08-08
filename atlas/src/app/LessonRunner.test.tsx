import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";

import { AtlasApp } from "./AtlasApp";

describe("comparison lesson runner", () => {
  beforeEach(() => {
    window.location.hash = "#lessons/adas-vs-aflow";
  });

  it("requires a prediction before revealing the comparison", async () => {
    const user = userEvent.setup();
    render(<AtlasApp />);

    expect(
      screen.getByRole("heading", { name: "ADAS versus AFlow" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("textbox", { name: "Your prediction" }))
      .toBeInTheDocument();
    expect(screen.queryByRole("heading", { name: "Compare" }))
      .not.toBeInTheDocument();

    await user.type(
      screen.getByRole("textbox", { name: "Your prediction" }),
      "AFlow searches complete workflows with MCTS.",
    );
    await user.click(screen.getByRole("button", { name: "Reveal comparison" }));

    expect(screen.getByRole("heading", { name: "Compare" })).toBeInTheDocument();
    expect(screen.getAllByText("class.harness-improvement")).toHaveLength(2);
  });

  it("transfers a compared case into diagnosis", async () => {
    const user = userEvent.setup();
    render(<AtlasApp />);

    await user.click(screen.getByRole("button", { name: "Diagnose AFlow" }));

    expect(window.location.hash).toBe("#diagnose/aflow");
    expect(screen.getByRole("heading", { name: "AFlow diagnosis" }))
      .toBeInTheDocument();
  });

  it("starts a clean lesson when the component is remounted", async () => {
    const user = userEvent.setup();
    const first = render(<AtlasApp />);
    await user.type(
      screen.getByRole("textbox", { name: "Your prediction" }),
      "Prediction.",
    );
    await user.click(screen.getByRole("button", { name: "Reveal comparison" }));
    expect(screen.getByRole("heading", { name: "Compare" })).toBeInTheDocument();

    first.unmount();
    render(<AtlasApp />);

    expect(screen.queryByRole("heading", { name: "Compare" }))
      .not.toBeInTheDocument();
  });
});
