
<template>
  <span class="p-1 rounded-sm hover:bg-accent bg-opacity-25">
    <svg
      class="text-blue-500 hover:text-white fill-current inline"
      focusable="false"
      width="1em"
      height="1em"
      viewBox="0 0 24 24"
    >
      <path
        d="M14 11a3 3 0 0 1-3-3V4H7a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2h9a2 2 0 0 0 2-2v-8h-4zm-2-3a2 2 0 0 0 2 2h3.586L12 4.414V8zM7 3h5l7 7v9a3 3 0 0 1-3 3H7a3 3 0 0 1-3-3V6a3 3 0 0 1 3-3z"
        fill="currentColor"
      ></path>
    </svg>
    {{ node.title }}
  </span>
</template>

<script>
export default {
  name: "MdiLightFile",
  props: {
    node: Object,
  },
};
</script>