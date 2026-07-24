<!-- Atmospheric backdrop: evokes the "frosted glass over the game world" feel
     without a live game behind us. Layered gradients + a few drifting glow motes. -->
<div class="backdrop" aria-hidden="true">
  <span class="mote m1"></span>
  <span class="mote m2"></span>
  <span class="mote m3"></span>
  <span class="mote m4"></span>
  <span class="mote m5"></span>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: -1;
    overflow: hidden;
    background:
      radial-gradient(1200px 700px at 18% -8%, rgba(176, 96, 224, 0.18), transparent 60%),
      radial-gradient(1100px 800px at 92% 108%, rgba(63, 199, 224, 0.12), transparent 55%),
      radial-gradient(900px 600px at 60% 40%, rgba(30, 42, 58, 0.5), transparent 70%),
      var(--app-bg);
  }
  .mote {
    position: absolute;
    border-radius: 50%;
    background: rgba(176, 96, 224, 0.7);
    box-shadow: 0 0 8px 2px rgba(176, 96, 224, 0.5);
    animation: floatMote 11s ease-in-out infinite;
  }
  .m1 { left: 14%; top: 22%; width: 3px; height: 3px; }
  .m2 { left: 32%; top: 66%; width: 2px; height: 2px; background: rgba(63, 199, 224, 0.7); box-shadow: 0 0 7px 2px rgba(63, 199, 224, 0.5); animation-duration: 13s; animation-delay: 0.8s; }
  .m3 { left: 71%; top: 18%; width: 2px; height: 2px; background: rgba(245, 166, 35, 0.55); box-shadow: 0 0 7px 2px rgba(245, 166, 35, 0.4); animation-duration: 12s; animation-delay: 1.6s; }
  .m4 { left: 84%; top: 72%; width: 3px; height: 3px; background: rgba(176, 96, 224, 0.6); animation-duration: 14s; animation-delay: 0.4s; }
  .m5 { left: 52%; top: 12%; width: 2px; height: 2px; background: rgba(63, 199, 224, 0.55); box-shadow: 0 0 6px 2px rgba(63, 199, 224, 0.4); animation-duration: 10s; animation-delay: 2.1s; }
  @keyframes floatMote {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-24px); }
  }
</style>
