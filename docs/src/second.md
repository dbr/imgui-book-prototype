# The Second

Simplest example, display code and render output

```imgui-example
use imgui::im_str;
ui.text(im_str!("Basic example"))
```

Hide code, but display resulting image:
```imgui-example,hide_code
use imgui::im_str;
ui.text(im_str!("Example with no code visible!"))
```

Create image but don't display it
```imgui-example,hide_output
use imgui::im_str;
ui.text(im_str!("Only the code is visible, but an image is output"))
```

Create image but don't display code or image:
```imgui-example,hide
use imgui::im_str;
ui.text(im_str!("Not visible at all"))
```

Compile but don't actually run example:
```imgui-example,no_run
use imgui::im_str;
ui.text(im_str!("This is never run, but the code is visible"))
```

Compile and run, but expect a panic:
```imgui-example,should_panic
use imgui::im_str;
ui.text(im_str!("Hello world!"));
panic!();
```

Don't compile or run this:
```imgui-example,ignore
use imgui::im_str;
ui.text(im_str!("This is not run at all!"));
so we can do nonsense like this, which isn't valid Rust at all
```

## Named blocks

Blocks can be named, and an labelled link will be created so the image
can be referred elsewhere in the document:

```imgui-example,name=basic-button
use imgui::im_str;
if ui.button(im_str!("Hi")) {
    ui.text(im_str!("Clicked!"));
}
```

Same image again:

![image][basic-button]


More usefully, we can hide the output and display it elsewhere where in the document. There is a hidden code block here.

```imgui-example,name=more-useful,hide_output
use imgui::im_str;
if ui.button(im_str!("Hi")) {
    ui.text(im_str!("Clicked!"));
}
```

And the output is displayed shortly after:
![The output][more-useful]

Similarly we could use `hide` to hide the code block also.


## Hiding lines

Same as rustdoc, we don't display lines prefixed with `#`

```imgui-example
# use imgui::im_str;
# imgui::Window::new(im_str!("Example")).build(&ui, || {
ui.text(im_str!("Clicked!"));
# })
```
