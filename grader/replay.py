"""SSIM frame-replay grader — Phase 6.

Computes the structural similarity between two RGB framebuffers — the "replay"
signal of GBA Eval's grading (60% weight per ADR-003). SSIM ≈ 1.0 for
near-identical frames, lower for diverging frames.

Dependency-free (pure Python) to match the project's "dependency-light" spirit.
"""

from typing import Sequence

# Standard SSIM constants for 8-bit dynamic range (L = 255): C1 = (0.01*L)^2,
# C2 = (0.03*L)^2 (Wang et al., 2004).
C1 = (0.01 * 255) ** 2
C2 = (0.03 * 255) ** 2


def ssim(a: Sequence[int], b: Sequence[int], c1: float = C1, c2: float = C2) -> float:
    """Global SSIM between two equal-length 8-bit sequences.

    Uses the luminance + contrast + structure formulation, computed globally
    (no sliding window — sufficient for a per-frame replay comparison).
    """
    n = len(a)
    if n == 0 or n != len(b):
        raise ValueError("non-empty, equal-length sequences required")

    mu_a = sum(a) / n
    mu_b = sum(b) / n
    var_a = sum((x - mu_a) ** 2 for x in a) / (n - 1)
    var_b = sum((x - mu_b) ** 2 for x in b) / (n - 1)
    cov = sum((a[i] - mu_a) * (b[i] - mu_b) for i in range(n)) / (n - 1)

    num = (2 * mu_a * mu_b + c1) * (2 * cov + c2)
    den = (mu_a * mu_a + mu_b * mu_b + c1) * (var_a + var_b + c2)
    return num / den


if __name__ == "__main__":
    a = [255] * 100
    b = list(a)
    c = [0] * 100

    assert abs(ssim(a, b) - 1.0) < 1e-6, "identical frames should give SSIM 1.0"
    assert ssim(a, c) < 1.0, "different frames should give SSIM < 1.0"

    print(f"identical SSIM        = {ssim(a, b):.6f}")
    print(f"red-vs-black SSIM     = {ssim(a, c):.6f}")
    print("OK")
