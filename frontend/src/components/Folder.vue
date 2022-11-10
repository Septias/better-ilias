<script lang="ts" setup>
import type { PropType } from 'vue'
import { ref } from 'vue'
import type { IlNode } from '~/types'

const props = defineProps({
  node: {
    type: Object as PropType<IlNode>,
    required: true,
  },
  index: {
    type: Number,
    required: true,
  },
})

const emit = defineEmits(['setVisible', 'setInvisible'])
const expanded = ref(false)

function handle_click() {
  if (edit_visibility.value) {
    if (props.node.visible) {
      emit('setVisible', [props.index])
    }
    else {
      emit('setInvisible', [props.index])
    }
  }
  else {
    expanded.value = !expanded.value
  }
}

function open_folder() {
  // ws open folder
}

function open_page() {
  window.open(`https://ilias.uni-freiburg.de/${props.node.uri}`)
}

const folder_icon = computed(() => {
  if (props.node.children && props.node.children.length) {
    if (expanded.value) {
      return ['i-ic-outline-keyboard-arrow-down']
    }
    else {
      return ['i-ic-outline-keyboard-arrow-right']
    }
  }
  else {
    return ['i-ic-round-grid-3x3']
  }
})
</script>

<template lang="pug">
div(v-if='node.visible || edit_visibility')
  div.flex.items-center
    div.text-accent(:class="folder_icon" @click="handle_click")
    span.text-white.no-select(
      :class="{ 'text-opacity-25': !node.visible && edit_visibility }"
      @click.exact='handle_click'
      @click.ctrl.exact='open_page'
      @click.shift.exact='open_folder'
      )
      span.p-1.rounded-sm.no-select(class='hover:bg-light_main') {{ node.title }}
    // .text-accent(@click='() => {activate_note(node);}')
  dynamic-children(:children="node.children" :index="index" v-if="expanded")
</template>

<style lang="sass">
.no-select
  --webkit-user-select: none
  user-select: none
</style>
