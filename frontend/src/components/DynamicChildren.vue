<script lang="ts" setup>
import type { PropType } from 'vue'

import File from './File.vue'
import DirectLink from './DirectLink.vue'
import Forum from './Forum.vue'
import Video from './Video.vue'
import Exercise from './Exercise.vue'
import Folder from './Folder.vue'
import type { IlNode } from '~/types'
import { get_breed } from '~/utils'

const props = defineProps({
  children: Array as PropType<IlNode[]>,
  index: {
    type: Number,
    required: true,
  },
})

const emit = defineEmits(['setVisible', 'setInvisible'])

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
    case 'Folder':
      return Folder
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
</script>

<template lang="pug">
ul.node_tree
  li.node_tree_item(
    v-for='(child, index) in children'
    :key='child.id'
    :index='index'
    :class='[child.breed]'
    )
      component(
        :is='get_component(child.breed)'
        :node='child'
        :index='index'
        @set_invisible='handle_set_inivisible'
        @set_visible='handle_set_visible'
        )
</template>

<style lang="sass">
.node_tree
  list-style: none
  padding-inline-start: 20px
  overflow: hidden
  max-height: 100%
  transition: all 5s ease
.node_tree_item
  cursor: pointer
</style>
