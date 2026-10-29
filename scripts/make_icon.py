from PIL import Image, ImageDraw

SIZE = 1024
SCALE = 4


def lerp(a, b, t):
    return tuple(round(x + (y - x) * t) for x, y in zip(a, b))


def main():
    big = SIZE * SCALE
    img = Image.new("RGBA", (big, big), (0, 0, 0, 0))

    # Diagonal gradient background.
    grad = Image.new("RGB", (big, big))
    pixels = grad.load()
    top = (8, 145, 178)  # cyan-600
    bottom = (15, 23, 42)  # slate-900
    for y in range(big):
        for x in range(big):
            t = (x + y) / (2 * (big - 1))
            pixels[x, y] = lerp(top, bottom, t)

    # Rounded-square mask.
    mask = Image.new("L", (big, big), 0)
    ImageDraw.Draw(mask).rounded_rectangle(
        (0, 0, big - 1, big - 1), radius=big * 0.22, fill=255
    )
    img.paste(grad, (0, 0), mask)

    draw = ImageDraw.Draw(img)
    cx = cy = big // 2
    ring_radius = big * 0.315
    ring_width = round(big * 0.055)

    # Soft outer glow.
    for i, alpha in ((3, 18), (2, 28), (1, 40)):
        grow = ring_width * i
        draw.ellipse(
            (
                cx - ring_radius - grow,
                cy - ring_radius - grow,
                cx + ring_radius + grow,
                cy + ring_radius + grow,
            ),
            outline=(34, 211, 238, alpha),
            width=round(ring_width * 1.4),
        )

    draw.ellipse(
        (
            cx - ring_radius,
            cy - ring_radius,
            cx + ring_radius,
            cy + ring_radius,
        ),
        outline=(34, 211, 238, 255),
        width=ring_width,
    )

    hand_width = round(big * 0.042)
    draw.line((cx, cy, cx, cy - ring_radius * 0.62), fill=(224, 242, 254, 255), width=hand_width)
    draw.line((cx, cy, cx + ring_radius * 0.48, cy + ring_radius * 0.36), fill=(224, 242, 254, 255), width=hand_width)
    dot = round(big * 0.032)
    draw.ellipse((cx - dot, cy - dot, cx + dot, cy + dot), fill=(34, 211, 238, 255))

    img = img.resize((SIZE, SIZE), Image.LANCZOS)
    img.save("src-tauri/app-icon.png")
    print("saved src-tauri/app-icon.png")


if __name__ == "__main__":
    main()
