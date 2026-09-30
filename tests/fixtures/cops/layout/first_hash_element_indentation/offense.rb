func({
  a: 1,
  ^^^^ Layout/FirstHashElementIndentation: Use 2 spaces for indentation in a hash, relative to the first position after the preceding left parenthesis.
  b: 2
})
^ Layout/FirstHashElementIndentation: Indent the right brace the same as the first position after the preceding left parenthesis.

var = {
        a: 1
        ^^^^ Layout/FirstHashElementIndentation: Use 2 spaces for indentation in a hash, relative to the start of the line where the left curly brace is.
      }
      ^ Layout/FirstHashElementIndentation: Indent the right brace the same as the start of the line where the left brace is.

a << {
  }
  ^ Layout/FirstHashElementIndentation: Indent the right brace the same as the start of the line where the left brace is.

func(x: {
      a: 1, b: 2 })
      ^^^^ Layout/FirstHashElementIndentation: Use 2 spaces for indentation in a hash, relative to the first position after the preceding left parenthesis.

func(x: {
  a: 1,
  ^^^^ Layout/FirstHashElementIndentation: Use 2 spaces for indentation in a hash, relative to the parent hash key.
       b: 2
},
^ Layout/FirstHashElementIndentation: Indent the right brace the same as the parent hash key.
     y: {
       c: 1
     })

func x, {
       a: 1, b: 2 }
       ^^^^ Layout/FirstHashElementIndentation: Use 2 spaces for indentation in a hash, relative to the start of the line where the left curly brace is.

super({
       a: 1
       ^^^^ Layout/FirstHashElementIndentation: Use 2 spaces for indentation in a hash, relative to the start of the line where the left curly brace is.
     })
     ^ Layout/FirstHashElementIndentation: Indent the right brace the same as the start of the line where the left brace is.

expect(described_class.call(products: Product.actual,
                            selected_filters:,
                            shop:,
                            online: true,
                            category:)).to eq({
                                                 filters: [
                                                 ^^^^^^^ Layout/FirstHashElementIndentation: Use 2 spaces for indentation in a hash, relative to the first position after the preceding left parenthesis.
                                                 ],
                                                 count: 1
                                               })
                                               ^ Layout/FirstHashElementIndentation: Indent the right brace the same as the first position after the preceding left parenthesis.
