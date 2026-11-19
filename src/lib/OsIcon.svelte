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
  // Les logos très sombres (Apple, AlmaLinux…) deviendraient invisibles sur fond sombre : couleur du texte à la place.
  let color = $derived.by(() => {
    if (!icon) return 'currentColor';
    const n = parseInt(icon.hex, 16);
    const lum = (0.299 * (n >> 16) + 0.587 * ((n >> 8) & 255) + 0.114 * (n & 255)) / 255;
    return lum < 0.28 ? 'var(--text)' : `#${icon.hex}`;
  });
</script>

{#if icon}
  <svg width={size} height={size} viewBox="0 0 24 24" fill={color} aria-hidden="true"><path d={icon.path} /></svg>
{/if}
