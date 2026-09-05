import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";
import { AtlasApp } from "./AtlasApp";

describe("research-first Atlas", () => {
  beforeEach(() => { window.location.hash = ""; });

  it("orients a new reader and opens the execution reading path", async () => {
    const user = userEvent.setup();
    render(<AtlasApp />);
    expect(screen.getByRole("heading", {
      name: "How can agents improve, and how can we verify it?",
    })).toBeInTheDocument();
    expect(screen.queryByRole("navigation", { name: "Weng article sections" }))
      .not.toBeInTheDocument();
    await user.click(screen.getByRole("link", { name: "Execution" }));
    expect(window.location.hash).toBe("#explore/execution");
    expect(await screen.findByRole("heading", { name: "Durable execution" }))
      .toBeInTheDocument();
    await user.click(screen.getByRole("link", { name: "Understand Harp's executor" }));
    expect(await screen.findByRole("heading", { name: "Harp's executor and scheduler" }))
      .toHaveFocus();
    expect(screen.getByText(/Compile and validate TaskGraph/)).toBeInTheDocument();
  });

  it("skips to the current reading without replacing its route", async () => {
    const user = userEvent.setup();
    window.location.hash = "#explore/mathematics";
    render(<AtlasApp />);
    await user.click(screen.getByRole("link", { name: "Skip to content" }));
    expect(screen.getByRole("main")).toHaveFocus();
    expect(window.location.hash).toBe("#explore/mathematics");
  });
});
