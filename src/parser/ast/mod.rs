
trait AstInfo {

}

trait AstNode<I: AstInfo> {
    fn ast_info(&self) -> &I;
    fn traverse<F: FnMut(&dyn AstNode<I>)>(&self, _f: F) where Self: Sized {
        todo!()
    }
}
