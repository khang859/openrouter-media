/** One image option as text: `resolution: 1K, 2K, 4K`, `n: 1-4`, or just `seed`. */
function describeParameter([name, spec]) {
  if (spec.type === 'enum') return `${name}: ${spec.values.join(', ')}`;
  if (spec.type === 'range') return `${name}: ${spec.min}-${spec.max}`;
  return name;
}

function describeImageModel(model) {
  return Object.entries(model.supported_parameters ?? {}).map(describeParameter);
}

function describeVideoModel(model) {
  const list = (label, values, unit = '') =>
    Array.isArray(values) && values.length > 0 ? [`${label}: ${values.join(', ')}${unit}`] : [];
  return [
    ...list('duration', model.supported_durations, ' s'),
    ...list('resolution', model.supported_resolutions),
    ...list('aspect_ratio', model.supported_aspect_ratios),
    ...list('frames', model.supported_frame_images),
    ...(model.generate_audio ? ['audio'] : []),
    ...(model.seed ? ['seed'] : []),
    // Shown raw: the price keys differ per provider and say their own unit.
    ...Object.entries(model.pricing_skus ?? {}).map(([sku, price]) => `price ${sku}: ${price}`)
  ];
}

/** A model as a few lines of text: its id and name, then one option per line. */
export function describeModel(kind, model) {
  const lines = kind === 'video' ? describeVideoModel(model) : describeImageModel(model);
  return [`${model.id} (${model.name})`, ...lines.map((line) => `  ${line}`)].join('\n');
}
