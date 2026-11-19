<script lang="ts">
  import {
    siAlmalinux, siAlpinelinux, siApple, siArchlinux, siCentos, siDebian, siElementary, siFedora, siFreebsd, siGentoo,
    siKalilinux, siLinux, siLinuxmint, siManjaro, siNixos, siOpenwrt, siOpensuse, siPopos, siRaspberrypi, siRedhat,
    siRockylinux, siUbuntu, siZorin,
  } from 'simple-icons';

  // Logos de distributions (simple-icons, CC0). Les marques appartiennent à leurs propriétaires.
  const icons: Record<string, { path: string; hex: string }> = {
    debian: siDebian, ubuntu: siUbuntu, fedora: siFedora, arch: siArchlinux, centos: siCentos, rhel: siRedhat,
    alpine: siAlpinelinux, opensuse: siOpensuse, linuxmint: siLinuxmint, raspberrypi: siRaspberrypi, rocky: siRockylinux,
    alma: siAlmalinux, kali: siKalilinux, manjaro: siManjaro, gentoo: siGentoo, nixos: siNixos, popos: siPopos,
    freebsd: siFreebsd, apple: siApple, openwrt: siOpenwrt, zorin: siZorin, elementary: siElementary, linux: siLinux,
  };

  let { os, size = 20 }: { os: string | null | undefined; size?: number } = $props();

  let icon = $derived(os ? icons[os] : undefined);
  // Fond sur lequel l'icône est posée (carte / onglet) : on éclaircit la couleur de marque si le contraste est trop faible.
  const BG = [24, 28, 42];
  const lin = (c: number) => ((c /= 255) <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
  const lum = (r: number, g: number, b: number) => 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
  const contrast = (c: number[]) => (lum(...(c as [number, number, number])) + 0.05) / (lum(BG[0], BG[1], BG[2]) + 0.05);

  let color = $derived.by(() => {
    if (!icon) return 'currentColor';
    const n = parseInt(icon.hex, 16);
    const base = [n >> 16, (n >> 8) & 255, n & 255];
    for (let t = 0; t <= 1.001; t += 0.04) {
      const c = base.map((v) => Math.round(v + (255 - v) * t));
      if (contrast(c) >= 3.4) return `rgb(${c[0]} ${c[1]} ${c[2]})`;
    }
    return '#fff';
  });
</script>

{#if icon}
  <svg width={size} height={size} viewBox="0 0 24 24" fill={color} aria-hidden="true"><path d={icon.path} /></svg>
{/if}
