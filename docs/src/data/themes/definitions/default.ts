import type {ThemeDefinition} from '../types';

const def: ThemeDefinition = {
  id: 'default',
  name: 'AMUD Default',
  description:
    'The built-in orange glass cockpit look. No custom CSS required — reset the Custom CSS field or leave it empty.',
  tags: ['default', 'orange', 'glass', 'built-in'],
  category: 'default',
  cssFile: '',
  previewImage: 'img/AMUD-Dashboard.png',
  bundled: false,
  palette: {
    background: '#0b0e14',
    card: 'rgba(15,20,25,0.45)',
    accent: '#cf6427',
    text: '#f8fafc',
  },
};

export default def;
