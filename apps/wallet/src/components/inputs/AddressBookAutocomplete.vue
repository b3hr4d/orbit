<template>
  <VAutocomplete
    v-model="model"
    :multiple="props.multiple.value"
    :label="props.label.value"
    :custom-filter="() => true"
    auto-select-first
    item-value="id"
    item-title="address_owner"
    return-object
    clear-on-select
    hide-selected
    :items="items"
    :variant="props.variant.value"
    :density="props.density.value"
    :readonly="props.readonly.value"
    :disabled="props.disabled.value"
    @update:search="autocomplete.searchItems($event)"
  >
    <template #item="{ props: itemProps, item }">
      <!-- prettier-ignore -->
      <VListItem @click="(itemProps.onClick as any)">
        <VListItemTitle>{{ item.raw.address_owner }}</VListItemTitle>
        <VListItemSubtitle>{{ item.raw.address }}</VListItemSubtitle>
      </VListItem>
    </template>
  </VAutocomplete>
</template>
<script setup lang="ts">
import { computed, onMounted, ref, toRefs, watch } from 'vue';
import { useAddressBookAutocomplete } from '~/composables/autocomplete.composable';
import { AddressBookEntry } from '~/generated/station/station.did';

const input = withDefaults(
  defineProps<{
    modelValue?: AddressBookEntry[] | AddressBookEntry;
    label?: string;
    variant?: 'underlined' | 'outlined';
    density?: 'comfortable' | 'compact';
    multiple?: boolean;
    readonly?: boolean;
    disabled?: boolean;
  }>(),
  {
    modelValue: () => [],
    label: undefined,
    variant: 'underlined',
    density: 'comfortable',
    multiple: false,
    readonly: false,
    disabled: false,
  },
);

const props = toRefs(input);

const emit = defineEmits<{
  (event: 'update:modelValue', payload: AddressBookEntry[] | AddressBookEntry): void;
}>();

const model = computed({
  get: () => props.modelValue.value,
  set: value => emit('update:modelValue', value),
});

const items = ref<AddressBookEntry[]>([]);
const autocomplete = useAddressBookAutocomplete();

onMounted(() => {
  autocomplete.searchItems();
});

watch(
  () => autocomplete.results.value,
  results => {
    items.value = results;
  },
);
</script>
