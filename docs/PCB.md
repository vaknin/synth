SKiDL → KiCad → FreeRouting Flow (Open‑Source)

- Goal: go from SKiDL netlist to fabricated PCB using KiCad, autoroute with FreeRouting, then
generate fab/assembly outputs. KiKit is optional for panelization.

Prerequisites

- SKiDL (Python) for schematic/netlist generation.
- KiCad (v7/8) with kicad-cli.
- FreeRouting (standalone or integrated).
- Optional: KiKit for panelization; KiAuto/tracespace for automation/previews.

Process

- Capture: write circuit in SKiDL; generate KiCad‑compatible netlist.
- KiCad setup: create project/PCB, set board outline, stackup, and design rules to match
your fab.
- Footprints: assign in SKiDL or within KiCad; annotate as needed.
- Placement: place components on the PCB manually (recommended).
- DSN export: File → Export → Spectra DSN… from KiCad PCB Editor.
- FreeRouting: import DSN, autoroute under your rules, export *.ses.
- Import SES: File → Import → Spectra Session… back into KiCad; review and touch‑up routes.
- Finalize: add zones (GND pours), stitching vias, silks/text, keepouts; rerun DRC until
clean.
- Panelization (optional): if you need arrays/rails/mouse‑bites, use KiKit; otherwise skip.
- Outputs: generate Gerbers/Drills (and optionally stencil, positions, BOM) for fabrication/
assembly.

When To Use KiKit

- Use KiKit only if you want to send a pre‑panelized board. Many fabs will panelize for you
from a single‑board Gerber set.

Fabrication Outputs (typical)

- Gerber RS‑274X: F.Cu, B.Cu (and internal layers if 4+), F.Mask, B.Mask, F.SilkS, B.SilkS,
Edge.Cuts
- Drill (Excellon): plated and NPTH *.drl (separate or combined)
- Stencil (optional): F.Paste (and B.Paste if double‑sided)
- Assembly (optional): position/centroid files per side, BOM (CSV)

KiCad CLI Examples

# Gerbers
kicad-cli pcb export gerbers project.kicad_pcb -o out/gerbers \
--layers F.Cu,B.Cu,F.Mask,B.Mask,F.SilkS,B.SilkS,Edge.Cuts

# Drills
kicad-cli pcb export drill project.kicad_pcb -o out/drill --format excellon

# Positions (centroids) for assembly
kicad-cli pcb export pos project.kicad_pcb -o out/pos --format csv --units mm

# BOM (if using a KiCad schematic)
kicad-cli sch export bom project.kicad_sch -o out/bom.csv

Notes

- Set track/clearance/via rules to your fab’s minima (e.g., 6/6 mil; 0.3/0.6 mm vias) and
confirm with their capabilities.
- Keep copper ≥0.25–0.5 mm from Edge.Cuts; avoid silkscreen on exposed pads.
- Ground pours and proper return paths reduce noise; add fiducials/testpoints if assembling.

