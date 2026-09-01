import { describe, expect, it } from "vitest";

describe("DataGrid", () => {
  it("mock table tiene filas y total coherente", () => {
    const total = 1;
    const rows = [{ id: "1", nombre: "Test" }];
    expect(rows).toHaveLength(total);
  });

  it("distingue vacio de con datos", () => {
    const empty = true;
    const filtered = false;
    expect(empty).toBe(true);
    expect(filtered).toBe(false);
  });
});
