<script lang="ts">
  // Jeu d'icônes en trait (24x24), cohérent et sans dépendance.
  const paths: Record<string, string> = {
    server: 'M4 5h16v6H4zM4 13h16v6H4zM8 8h.01M8 16h.01',
    plus: 'M12 5v14M5 12h14',
    x: 'M6 6l12 12M18 6L6 18',
    play: 'M7 5l12 7-12 7z',
    folder: 'M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z',
    file: 'M6 3h8l4 4v14H6zM14 3v4h4',
    terminal: 'M4 5h16v14H4zM8 10l3 2-3 2M13 15h4',
    macro: 'M13 3L5 14h6l-1 7 8-11h-6z',
    record: 'M12 12m-5 0a5 5 0 1 0 10 0a5 5 0 1 0-10 0',
    stop: 'M7 7h10v10H7z',
    search: 'M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14zM20 20l-4-4',
    edit: 'M4 20h4L19 9l-4-4L4 16zM13 7l4 4',
    trash: 'M5 7h14M10 7V4h4v3M7 7l1 13h8l1-13M10 11v6M14 11v6',
    copy: 'M9 9h11v11H9zM5 15V4h11',
    up: 'M12 19V5M6 11l6-6 6 6',
    down: 'M12 5v14M6 13l6 6 6-6',
    download: 'M12 4v11M7 11l5 5 5-5M5 20h14',
    upload: 'M12 16V5M7 9l5-5 5 5M5 20h14',
    refresh: 'M20 11a8 8 0 1 0-2.3 6M20 4v7h-7',
    home: 'M4 11l8-7 8 7M6 10v10h12V10',
    chevron: 'M9 6l6 6-6 6',
    link: 'M10 14a4 4 0 0 0 5.6 0l3-3a4 4 0 0 0-5.6-5.6l-1 1M14 10a4 4 0 0 0-5.6 0l-3 3a4 4 0 0 0 5.6 5.6l1-1',
    key: 'M14 10a4 4 0 1 0-3.9 4H11l1 1h2v2h2v2h3v-3l-5.3-5.3M8 10h.01',
    archive: 'M4 5h16v4H4zM6 9v10h12V9M10 13h4',
    settings: 'M12 9a3 3 0 1 0 0 6 3 3 0 0 0 0-6zM19 12l2-1-1-3-2 .5-1.5-1.5L17 4l-3-1-1 2h-2L10 3 7 4l.5 2.5L6 8l-2-.5-1 3 2 1v2l-2 1 1 3 2-.5L7.5 17 7 19.5l3 1 1-2h2l1 2 3-1-.5-2.5L18 15.5l2 .5 1-3-2-1z',
    folderPlus: 'M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2zM12 10v6M9 13h6',
    check: 'M5 12l5 5 9-10',
    alert: 'M12 4l9 16H3zM12 10v4M12 17h.01',
    code: 'M8 8l-5 4 5 4M16 8l5 4-5 4M14 5l-4 14',
    globe: 'M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18zM3 12h18M12 3c3 3 3 15 0 18M12 3c-3 3-3 15 0 18',
    plug: 'M9 3v5M15 3v5M6 8h12v4a6 6 0 0 1-12 0zM12 18v3',
    info: 'M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18zM12 11v5M12 8h.01',
    more: 'M5 12h.01M12 12h.01M19 12h.01',
  };
  let { name, size = 16 }: { name: string; size?: number } = $props();
</script>

<svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
  <path d={paths[name] ?? ''} />
</svg>
