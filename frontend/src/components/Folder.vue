<script lang="ts" setup>
import type { PropType } from 'vue'
import { ref } from 'vue'
import File from './File.vue'
import DirectLink from './DirectLink.vue'
import Forum from './Forum.vue'
import Video from './Video.vue'
import Exercise from './Exercise.vue'
import type { IlNode } from '~/types'
import { get_breed } from '~/utils'

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

function get_component(breed: any) {
  switch (get_breed(breed)) {
    case 'File':
      return File
    case 'DirectLink':
      return DirectLink
    case 'Forum':
      return Forum
    case 'Video':
      return Video
    case 'Exercise':
      return Exercise
    /* case 'Folder':
      return Folder */
    default:
      throw new Error('Unknown breed')
  }
}

function handle_set_inivisible(path: Array<Number>) {
  path.push(props.index)
  emit('setInvisible', path)
}
function handle_set_visible(path: Array<Number>) {
  path.push(props.index)
  emit('setVisible', path)
}
function open_page() {
  window.open(`https://ilias.uni-freiburg.de/${props.node.uri}`)
}

function activate_note(node: IlNode) {}
</script>

<template>
  <span v-if="node.visible || edit_visibility">
    <div class="pluslogic">
      <svg
        focusable="false"
        width="1em"
        height="1em"
        viewBox="0 0 24 24"
        class="text-accent hover:text-white fill-current inline"
        @click="expanded = !expanded"
      >
        <template v-if="node.children && node.children.length">
          <path
            v-if="expanded"
            d="M5.843 9.593L11.5 15.25l5.657-5.657l-.707-.707l-4.95 4.95l-4.95-4.95l-.707.707z"
            fill="currentColor"
          />
          <path
            v-else
            d="M8.593 18.157L14.25 12.5L8.593 6.843l-.707.707l4.95 4.95l-4.95 4.95l.707.707z"
            fill="currentColor"
          />
        </template>
        <rect
          v-else
          x="9"
          y="9"
          width="5"
          height="5"
          style="fill: rgba(0, 0, 0, 0); stroke-width: 2; stroke: currentColor"
        />
      </svg>
      <span
        class="text-white select-none"
        :class="{ 'text-opacity-25': !node.visible && edit_visibility }"
        @click.exact="handle_click"
        @click.ctrl.exact="open_page"
        @click.shift.exact="open_folder"
      >
        <span class="p-1 rounded-sm hover:bg-accent">
          {{ node.title }}
        </span>
      </span>

      <div
        class="plus text-accent"
        @click="
          () => {
            activate_note(node);
          }
        "
      />
    </div>

    <ul class="node_tree" :class="{ shrinked: !expanded }">
      <li
        v-for="(child, index) in node.children"
        :key="child.id"
        :index="index"
        class="node_tree_item" :class="[child.breed]"
      >
        <template v-if="get_breed(child.breed) !== 'Folder'">
          <component
            :is="get_component(child.breed)"
            v-if="get_breed(child.breed) !== 'Folder'"
            :node="child"
            :index="index"
            @set_invisible="handle_set_inivisible"
            @set_visible="handle_set_visible"
          />
        </template>
        <template v-else>
          <Folder
            v-if="get_breed(child.breed) === 'Folder'"
            :node="child"
            :index="index"
            @set_invisible="handle_set_inivisible"
            @set_visible="handle_set_visible"
          />
        </template>
      </li>
    </ul>
  </span>
</template>

<style lang="sass" scoped>
.pluslogic:hover
  > .plus
    display: inline-block !important

.pluslogic
  > .plus
    display: none

.node_tree
  list-style: none
  padding-inline-start: 20px
  max-height: 100%

.node_tree_item
  cursor: pointer

.shrinked
  max-height: 0px
  overflow: hidden
</style>
