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

  let { os, size = 36, title }: { os: string | null | undefined; size?: number; title?: string } = $props();

  let icon = $derived(os ? icons[os] : undefined);

  const lin = (c: number) => ((c /= 255) <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
  const lum = (n: number) => 0.2126 * lin(n >> 16) + 0.7152 * lin((n >> 8) & 255) + 0.0722 * lin(n & 255);

  // Pastille aux couleurs de la marque, logo blanc (ou sombre si la marque est claire, comme Linux en jaune).
  // Les marques quasi noires (Apple, AlmaLinux…) passent en pastille claire, illisibles sinon sur le thème sombre.
  let look = $derived.by(() => {
    if (!icon) return { bg: 'transparent', fg: 'currentColor' };
    const n = parseInt(icon.hex, 16);
    const l = lum(n);
    if (l < 0.05) return { bg: '#e8eaf5', fg: '#14161f' };
    return { bg: `#${icon.hex}`, fg: 1.05 / (l + 0.05) >= 2.2 ? '#ffffff' : '#14161f' };
  });
</script>

{#if icon}
  <span class="tile" style:width="{size}px" style:height="{size}px" style:background={look.bg} style:border-radius="{Math.round(size * 0.24)}px" {title}>
    <svg width={size * 0.62} height={size * 0.62} viewBox="0 0 24 24" fill={look.fg} aria-hidden="true"><path d={icon.path} /></svg>
  </span>
{/if}

<style>
  .tile { display: inline-grid; place-items: center; flex: none; }
</style>
