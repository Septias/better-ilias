
<template>
  <span class="p-1 rounded-sm hover:bg-accent bg-opacity-25">
    <svg
      class="text-yellow-500 hover:text-white fill-current inline"
      focusable="false"
      width="1em"
      height="1em"
      viewBox="0 0 24 24"
    >
      <path
        d="M2 16.586L5.586 13H15a2 2 0 0 0 2-2V6a2 2 0 0 0-2-2H4a2 2 0 0 0-2 2v10.586zM2 18H1V6a3 3 0 0 1 3-3h11a3 3 0 0 1 3 3v5a3 3 0 0 1-3 3H6l-4 4zm19 2.586V10a2 2 0 0 0-2-2V7a3 3 0 0 1 3 3v12h-1l-4-4H8a3 3 0 0 1-2.761-1.825l.797-.797A2 2 0 0 0 8 17h9.414L21 20.586z"
        fill="currentColor"
      ></path>
    </svg>
    {{ node.title }}
  </span>
</template>

<script>
export default {
  name: "Forum",
  props: {
    node: Object,
  },
};
</script>

